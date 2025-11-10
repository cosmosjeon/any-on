#![cfg(feature = "cloud")]

use std::{io, path::PathBuf};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
};
use db::models::project::{CreateProject, Project};
use deployment::Deployment;
use serde::Deserialize;
use serde_json::json;
use services::services::{
    git::GitService,
    github_service::{GitHubService, GitHubServiceError, RepositoryInfo},
};
use tokio::fs;
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{
    DeploymentImpl,
    routes::projects::{INVALID_PROJECT_NAME_MESSAGE, contains_invalid_project_name_chars},
};

#[derive(Debug, Deserialize, TS)]
pub struct CreateProjectFromGitHub {
    pub repository_id: i64,
    pub name: String,
    pub clone_url: String,
    pub setup_script: Option<String>,
    pub dev_script: Option<String>,
    pub cleanup_script: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RepositoryQuery {
    pub page: Option<u8>,
}

/// List GitHub repositories for the authenticated user
pub async fn list_repositories(
    State(deployment): State<DeploymentImpl>,
    Query(params): Query<RepositoryQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<RepositoryInfo>>>, StatusCode> {
    let page = params.page.unwrap_or(1);

    let github_token = match deployment.github_token().await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Ok(ResponseJson(ApiResponse::error(
                "GitHub token not configured. Please authenticate with GitHub first.",
            )));
        }
        Err(err) => {
            tracing::error!("Failed to load GitHub token: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create GitHub service with token
    let github_service = match GitHubService::new(&github_token) {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create GitHub service: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // List repositories
    match github_service.list_repositories(page).await {
        Ok(repositories) => {
            tracing::info!(
                "Retrieved {} repositories from GitHub (page {})",
                repositories.len(),
                page
            );
            Ok(ResponseJson(ApiResponse::success(repositories)))
        }
        Err(GitHubServiceError::TokenInvalid) => Ok(ResponseJson(ApiResponse::error(
            "GitHub token is invalid or expired. Please re-authenticate with GitHub.",
        ))),
        Err(e) => {
            tracing::error!("Failed to list GitHub repositories: {}", e);
            Ok(ResponseJson(ApiResponse::error(&format!(
                "Failed to retrieve repositories: {}",
                e
            ))))
        }
    }
}

/// Create a project from a GitHub repository
pub async fn create_project_from_github(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<CreateProjectFromGitHub>,
) -> Result<ResponseJson<ApiResponse<Project>>, StatusCode> {
    let repo_name = payload.name.trim();
    if repo_name.is_empty() {
        return Ok(ResponseJson(ApiResponse::error("Project name is required")));
    }
    if contains_invalid_project_name_chars(repo_name) {
        return Ok(ResponseJson(ApiResponse::error(
            INVALID_PROJECT_NAME_MESSAGE,
        )));
    }

    tracing::debug!("Creating project '{}' from GitHub repository", repo_name);

    // Get workspace path
    let workspace_path = match resolve_workspace_path(&deployment).await {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to get workspace path: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let target_path = workspace_path.join(repo_name);

    // Check if project directory already exists
    if target_path.exists() {
        return Ok(ResponseJson(ApiResponse::error(
            "A project with this name already exists in the workspace",
        )));
    }

    let repo_path_string = target_path.to_string_lossy().to_string();

    let mut tx = match deployment.db().pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to start project creation transaction: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let duplicate_for_user = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM projects WHERE user_id = ?1 AND git_repo_path = ?2",
    )
    .bind(deployment.user_id())
    .bind(&repo_path_string)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to check for existing git repo path: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if duplicate_for_user.is_some() {
        return Ok(ResponseJson(ApiResponse::error(
            "A project with this git repository path already exists",
        )));
    }

    // Get GitHub token
    let github_token = match deployment.github_token().await {
        Ok(Some(token)) => Some(token),
        Ok(None) => None,
        Err(err) => {
            tracing::error!("Failed to load GitHub token: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Clone the repository
    match GitService::clone_repository(&payload.clone_url, &target_path, github_token.as_deref()) {
        Ok(_) => {
            tracing::info!(
                "Successfully cloned repository {} to {}",
                payload.clone_url,
                target_path.display()
            );
        }
        Err(e) => {
            tracing::error!("Failed to clone repository: {}", e);
            return Ok(ResponseJson(ApiResponse::error(&format!(
                "Failed to clone repository: {}",
                e
            ))));
        }
    }

    // Create project record in database
    let has_setup_script = payload.setup_script.is_some();
    let has_dev_script = payload.dev_script.is_some();
    let project_data = CreateProject {
        name: repo_name.to_string(),
        git_repo_path: repo_path_string,
        use_existing_repo: true, // Since we just cloned it
        setup_script: payload.setup_script,
        dev_script: payload.dev_script,
        cleanup_script: payload.cleanup_script,
        copy_files: None,
    };

    let project_id = Uuid::new_v4();
    let project =
        match Project::create(&mut *tx, &project_data, project_id, deployment.user_id()).await {
            Ok(project) => {
                if let Err(e) = tx.commit().await {
                    tracing::error!("Failed to commit project creation transaction: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                project
            }
            Err(e) => {
                tracing::error!("Failed to create project: {}", e);

                // Clean up cloned repository if project creation failed
                if target_path.exists() {
                    if let Err(cleanup_err) = std::fs::remove_dir_all(&target_path) {
                        tracing::error!("Failed to cleanup cloned repository: {}", cleanup_err);
                    }
                }

                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.message().contains(
                        "UNIQUE constraint failed: projects.user_id, projects.git_repo_path",
                    ) {
                        return Ok(ResponseJson(ApiResponse::error(
                            "A project with this git repository path already exists",
                        )));
                    }
                }

                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    deployment
        .track_if_analytics_allowed(
            "project_created",
            json!({
                "project_id": project.id.to_string(),
                "repository_id": payload.repository_id,
                "clone_url": payload.clone_url,
                "has_setup_script": has_setup_script,
                "has_dev_script": has_dev_script,
                "trigger": "github",
            }),
        )
        .await;

    Ok(ResponseJson(ApiResponse::success(project)))
}

/// Create router for GitHub-related endpoints (only registered in cloud mode)
pub fn github_router() -> Router<DeploymentImpl> {
    Router::new()
        .route("/github/repositories", get(list_repositories))
        .route("/projects/from-github", post(create_project_from_github))
}

async fn resolve_workspace_path(deployment: &DeploymentImpl) -> Result<PathBuf, io::Error> {
    let base_path = deployment
        .workspace_dir()
        .await
        .map_err(io::Error::from)?;
    fs::create_dir_all(&base_path).await?;
    Ok(base_path)
}
