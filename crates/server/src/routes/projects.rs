use std::{
    io,
    path::{Path, PathBuf},
};

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json as ResponseJson,
    routing::get,
};
use db::models::project::{
    CreateProject, Project, ProjectError, SearchMatchType, SearchResult, UpdateProject,
};
use deployment::Deployment;
use ignore::WalkBuilder;
use services::services::{
    file_ranker::FileRanker,
    file_search_cache::{CacheError, SearchMode, SearchQuery},
    git::GitBranch,
};
use tokio::fs;
use utils::{path::expand_tilde, response::ApiResponse};
use uuid::Uuid;

use crate::{
    DeploymentImpl, auth::AuthenticatedUser, error::ApiError, middleware::load_project_middleware,
};

pub(crate) const INVALID_PROJECT_NAME_CHARS: &[char] =
    &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
pub(crate) const INVALID_PROJECT_NAME_MESSAGE: &str =
    "Project name cannot contain / \\ : * ? \" < > | or control characters.";

pub(crate) fn contains_invalid_project_name_chars(name: &str) -> bool {
    name.chars()
        .any(|c| c.is_control() || INVALID_PROJECT_NAME_CHARS.contains(&c))
}

async fn resolve_workspace_path(deployment: &DeploymentImpl) -> Result<PathBuf, io::Error> {
    let base_path = deployment
        .workspace_dir()
        .await
        .map_err(io::Error::from)?;
    fs::create_dir_all(&base_path).await?;
    Ok(base_path)
}

/// Get all projects for the authenticated user
pub async fn get_projects(
    State(deployment): State<DeploymentImpl>,
    Extension(user): Extension<AuthenticatedUser>, // ✅ 추가
) -> Result<ResponseJson<ApiResponse<Vec<Project>>>, ApiError> {
    // ✅ find_by_user로 변경 (이 사용자의 프로젝트만)
    let projects = Project::find_by_user(&deployment.db().pool, &user.user_id).await?;

    tracing::debug!(
        "User {} retrieved {} projects",
        user.username,
        projects.len()
    );

    Ok(ResponseJson(ApiResponse::success(projects)))
}

pub async fn get_project(
    Extension(project): Extension<Project>,
) -> Result<ResponseJson<ApiResponse<Project>>, ApiError> {
    Ok(ResponseJson(ApiResponse::success(project)))
}

pub async fn get_project_branches(
    Extension(project): Extension<Project>,
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<GitBranch>>>, ApiError> {
    let branches = deployment.git().get_all_branches(&project.git_repo_path)?;
    Ok(ResponseJson(ApiResponse::success(branches)))
}

/// Create a new project for the authenticated user
pub async fn create_project(
    State(deployment): State<DeploymentImpl>,
    Extension(user): Extension<AuthenticatedUser>, // ✅ 추가
    Json(payload): Json<CreateProject>,
) -> Result<ResponseJson<ApiResponse<Project>>, ApiError> {
    let id = Uuid::new_v4();
    let CreateProject {
        name,
        git_repo_path,
        setup_script,
        dev_script,
        cleanup_script,
        copy_files,
        use_existing_repo,
    } = payload;
    tracing::debug!("Creating project '{}'", name);

    let name = name.trim().to_string();
    if name.is_empty() {
        return Ok(ResponseJson(ApiResponse::error("Project name is required")));
    }
    if contains_invalid_project_name_chars(&name) {
        return Ok(ResponseJson(ApiResponse::error(
            INVALID_PROJECT_NAME_MESSAGE,
        )));
    }

    // Replit-style: If git_repo_path is empty, auto-generate from workspace_dir + project name
    let path = if git_repo_path.is_empty() || git_repo_path.trim().is_empty() {
        // Get workspace directory (Replit-style)
        let workspace_path = resolve_workspace_path(&deployment)
            .await
            .map_err(|e| ApiError::Project(ProjectError::GitRepoCheckFailed(e.to_string())))?;
        workspace_path.join(&name)
    } else {
        std::path::absolute(expand_tilde(&git_repo_path))?
    };
    let path_string = path.to_string_lossy().to_string();

    let mut tx = deployment.db().pool.begin().await?;

    let existing_for_user = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM projects WHERE user_id = ?1 AND git_repo_path = ?2",
    )
    .bind(&user.user_id)
    .bind(&path_string)
    .fetch_optional(&mut *tx)
    .await?;

    if existing_for_user.is_some() {
        return Ok(ResponseJson(ApiResponse::error(
            "A project with this git repository path already exists",
        )));
    }

    if use_existing_repo {
        // For existing repos, validate that the path exists and is a git repository
        if !path.exists() {
            return Ok(ResponseJson(ApiResponse::error(
                "The specified path does not exist",
            )));
        }

        if !path.is_dir() {
            return Ok(ResponseJson(ApiResponse::error(
                "The specified path is not a directory",
            )));
        }

        if !path.join(".git").exists() {
            return Ok(ResponseJson(ApiResponse::error(
                "The specified directory is not a git repository",
            )));
        }

        // Ensure existing repo has a main branch if it's empty
        if let Err(e) = deployment.git().ensure_main_branch_exists(&path) {
            tracing::error!("Failed to ensure main branch exists: {}", e);
            return Ok(ResponseJson(ApiResponse::error(&format!(
                "Failed to ensure main branch exists: {}",
                e
            ))));
        }
    } else {
        // For new repos (Replit-style), create directory and initialize git
        // Create directory if it doesn't exist
        if !path.exists() {
            if let Err(e) = std::fs::create_dir_all(&path) {
                tracing::error!("Failed to create directory: {}", e);
                return Ok(ResponseJson(ApiResponse::error(&format!(
                    "Failed to create directory: {}",
                    e
                ))));
            }
        }

        // Check if it's already a git repo, if not initialize it
        if !path.join(".git").exists() {
            if let Err(e) = deployment.git().initialize_repo_with_main_branch(&path) {
                tracing::error!("Failed to initialize git repository: {}", e);
                return Ok(ResponseJson(ApiResponse::error(&format!(
                    "Failed to initialize git repository: {}",
                    e
                ))));
            }
        }
    }

    let create_payload = CreateProject {
        name,
        git_repo_path: path_string,
        use_existing_repo,
        setup_script,
        dev_script,
        cleanup_script,
        copy_files,
    };

    match Project::create(&mut *tx, &create_payload, id, &user.user_id).await {
        Ok(project) => {
            tx.commit().await?;
            // Track project creation event
            deployment
                .track_if_analytics_allowed(
                    "project_created",
                    serde_json::json!({
                        "project_id": project.id.to_string(),
                        "use_existing_repo": use_existing_repo,
                        "has_setup_script": project.setup_script.is_some(),
                        "has_dev_script": project.dev_script.is_some(),
                        "trigger": "manual",
                    }),
                )
                .await;

            Ok(ResponseJson(ApiResponse::success(project)))
        }
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err
                    .message()
                    .contains("UNIQUE constraint failed: projects.user_id, projects.git_repo_path")
                {
                    return Ok(ResponseJson(ApiResponse::error(
                        "A project with this git repository path already exists",
                    )));
                }
            }
            Err(ProjectError::CreateFailed(e.to_string()).into())
        }
    }
}

pub async fn update_project(
    Extension(existing_project): Extension<Project>,
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<UpdateProject>,
) -> Result<ResponseJson<ApiResponse<Project>>, StatusCode> {
    // Destructure payload to handle field updates.
    // This allows us to treat `None` from the payload as an explicit `null` to clear a field,
    // as the frontend currently sends all fields on update.
    let UpdateProject {
        name,
        git_repo_path,
        setup_script,
        dev_script,
        cleanup_script,
        copy_files,
    } = payload;
    let name = match name {
        Some(value) => {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                return Ok(ResponseJson(ApiResponse::error("Project name is required")));
            }
            if contains_invalid_project_name_chars(&trimmed) {
                return Ok(ResponseJson(ApiResponse::error(
                    INVALID_PROJECT_NAME_MESSAGE,
                )));
            }
            Some(trimmed)
        }
        None => None,
    };
    // If git_repo_path is being changed, check if the new path is already used by another project
    let git_repo_path = if let Some(new_git_repo_path) = git_repo_path.map(|s| expand_tilde(&s))
        && new_git_repo_path != existing_project.git_repo_path
    {
        match Project::find_by_git_repo_path_excluding_id(
            &deployment.db().pool,
            new_git_repo_path.to_string_lossy().as_ref(),
            existing_project.id,
        )
        .await
        {
            Ok(Some(_)) => {
                return Ok(ResponseJson(ApiResponse::error(
                    "A project with this git repository path already exists",
                )));
            }
            Ok(None) => new_git_repo_path,
            Err(e) => {
                tracing::error!("Failed to check for existing git repo path: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        existing_project.git_repo_path
    };

    match Project::update(
        &deployment.db().pool,
        existing_project.id,
        name.unwrap_or(existing_project.name),
        git_repo_path.to_string_lossy().to_string(),
        setup_script,
        dev_script,
        cleanup_script,
        copy_files,
    )
    .await
    {
        Ok(project) => Ok(ResponseJson(ApiResponse::success(project))),
        Err(e) => {
            tracing::error!("Failed to update project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_project(
    Extension(project): Extension<Project>,
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<()>>, StatusCode> {
    match Project::delete(&deployment.db().pool, project.id).await {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                deployment
                    .track_if_analytics_allowed(
                        "project_deleted",
                        serde_json::json!({
                            "project_id": project.id.to_string(),
                        }),
                    )
                    .await;

                Ok(ResponseJson(ApiResponse::success(())))
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn search_project_files(
    State(deployment): State<DeploymentImpl>,
    Extension(project): Extension<Project>,
    Query(search_query): Query<SearchQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<SearchResult>>>, StatusCode> {
    let query = search_query.q.trim();
    let mode = search_query.mode;

    if query.is_empty() {
        return Ok(ResponseJson(ApiResponse::error(
            "Query parameter 'q' is required and cannot be empty",
        )));
    }

    let repo_path = &project.git_repo_path;
    let file_search_cache = deployment.file_search_cache();

    // Try cache first
    match file_search_cache
        .search(repo_path, query, mode.clone())
        .await
    {
        Ok(results) => {
            tracing::debug!(
                "Cache hit for repo {:?}, query: {}, mode: {:?}",
                repo_path,
                query,
                mode
            );
            Ok(ResponseJson(ApiResponse::success(results)))
        }
        Err(CacheError::Miss) => {
            // Cache miss - fall back to filesystem search
            tracing::debug!(
                "Cache miss for repo {:?}, query: {}, mode: {:?}",
                repo_path,
                query,
                mode
            );
            match search_files_in_repo(&project.git_repo_path.to_string_lossy(), query, mode).await
            {
                Ok(results) => Ok(ResponseJson(ApiResponse::success(results))),
                Err(e) => {
                    tracing::error!("Failed to search files: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(CacheError::BuildError(e)) => {
            tracing::error!("Cache build error for repo {:?}: {}", repo_path, e);
            // Fall back to filesystem search
            match search_files_in_repo(&project.git_repo_path.to_string_lossy(), query, mode).await
            {
                Ok(results) => Ok(ResponseJson(ApiResponse::success(results))),
                Err(e) => {
                    tracing::error!("Failed to search files: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}

async fn search_files_in_repo(
    repo_path: &str,
    query: &str,
    mode: SearchMode,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let repo_path = Path::new(repo_path);

    if !repo_path.exists() {
        return Err("Repository path does not exist".into());
    }

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Configure walker based on mode
    let walker = match mode {
        SearchMode::Settings => {
            // Settings mode: Include ignored files but exclude performance killers
            WalkBuilder::new(repo_path)
                .git_ignore(false) // Include ignored files like .env
                .git_global(false)
                .git_exclude(false)
                .hidden(false)
                .filter_entry(|entry| {
                    let name = entry.file_name().to_string_lossy();
                    // Always exclude .git directories and performance killers
                    name != ".git"
                        && name != "node_modules"
                        && name != "target"
                        && name != "dist"
                        && name != "build"
                })
                .build()
        }
        SearchMode::TaskForm => {
            // Task form mode: Respect gitignore (cleaner results)
            WalkBuilder::new(repo_path)
                .git_ignore(true) // Respect .gitignore
                .git_global(true) // Respect global .gitignore
                .git_exclude(true) // Respect .git/info/exclude
                .hidden(false) // Still show hidden files like .env (if not gitignored)
                .filter_entry(|entry| {
                    let name = entry.file_name().to_string_lossy();
                    name != ".git"
                })
                .build()
        }
    };

    for result in walker {
        let entry = result?;
        let path = entry.path();

        // Skip the root directory itself
        if path == repo_path {
            continue;
        }

        let relative_path = path.strip_prefix(repo_path)?;
        let relative_path_str = relative_path.to_string_lossy().to_lowercase();

        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Check for matches
        if file_name.contains(&query_lower) {
            results.push(SearchResult {
                path: relative_path.to_string_lossy().to_string(),
                is_file: path.is_file(),
                match_type: SearchMatchType::FileName,
            });
        } else if relative_path_str.contains(&query_lower) {
            // Check if it's a directory name match or full path match
            let match_type = if path
                .parent()
                .and_then(|p| p.file_name())
                .map(|name| name.to_string_lossy().to_lowercase())
                .unwrap_or_default()
                .contains(&query_lower)
            {
                SearchMatchType::DirectoryName
            } else {
                SearchMatchType::FullPath
            };

            results.push(SearchResult {
                path: relative_path.to_string_lossy().to_string(),
                is_file: path.is_file(),
                match_type,
            });
        }
    }

    // Apply git history-based ranking
    let file_ranker = FileRanker::new();
    match file_ranker.get_stats(repo_path).await {
        Ok(stats) => {
            // Re-rank results using git history
            file_ranker.rerank(&mut results, &stats);
        }
        Err(e) => {
            tracing::warn!(
                "Failed to get git stats for ranking, using basic sort: {}",
                e
            );
            // Fallback to basic priority sorting
            results.sort_by(|a, b| {
                let priority = |match_type: &SearchMatchType| match match_type {
                    SearchMatchType::FileName => 0,
                    SearchMatchType::DirectoryName => 1,
                    SearchMatchType::FullPath => 2,
                };

                priority(&a.match_type)
                    .cmp(&priority(&b.match_type))
                    .then_with(|| a.path.cmp(&b.path))
            });
        }
    }

    // Limit to top 10 results
    results.truncate(10);

    Ok(results)
}

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    let project_id_router = Router::new()
        .route(
            "/",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route("/branches", get(get_project_branches))
        .route("/search", get(search_project_files))
        .layer(from_fn_with_state(
            deployment.clone(),
            load_project_middleware,
        ));

    let projects_router = Router::new()
        .route("/", get(get_projects).post(create_project))
        .nest("/{id}", project_id_router)
        // ✅ 모든 프로젝트 API에 인증 미들웨어 적용
        .layer(from_fn_with_state(
            deployment.clone(),
            crate::middleware::auth::require_auth,
        ));

    Router::new().nest("/projects", projects_router)
}
