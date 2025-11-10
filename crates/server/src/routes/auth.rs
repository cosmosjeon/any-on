use axum::{
    Json,
    Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{Json as ResponseJson, Response, Sse, sse::{Event, KeepAlive}},
    routing::{get, post},
};
use deployment::{Deployment, DeploymentError};
use octocrab::auth::Continue;
use serde::{Deserialize, Serialize};
use db::models::{
    draft::Draft,
    image::Image,
    project::Project,
    tag::Tag,
    task::Task,
    task_attempt::TaskAttempt,
};
use services::services::{
    auth::{AuthError, DeviceFlowStartResponse},
    config::save_config_to_file,
    github_service::{GitHubService, GitHubServiceError},
    secret_store::{SECRET_CLAUDE_ACCESS, SECRET_GITHUB_OAUTH},
};
use futures_util::StreamExt;
use uuid::Uuid;
use utils::response::ApiResponse;

use crate::{DeploymentImpl, error::ApiError};

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/auth/github/device/start", post(device_start))
        .route("/auth/github/device/poll", post(device_poll))
        .route("/auth/github/check", get(github_check_token))
        .route("/auth/claude/session", post(claude_session_start))
        .route(
            "/auth/claude/session/{session_id}/stream",
            get(claude_session_stream),
        )
        .route(
            "/auth/claude/session/{session_id}/input",
            post(claude_session_input),
        )
        .route(
            "/auth/claude/session/{session_id}/cancel",
            post(claude_session_cancel),
        )
        .route("/auth/claude/logout", post(claude_logout))
        .layer(from_fn_with_state(
            deployment.clone(),
            sentry_user_context_middleware,
        ))
}

/// POST /auth/github/device/start
async fn device_start(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<DeviceFlowStartResponse>>, ApiError> {
    let device_start_response = deployment.auth().device_start().await?;
    Ok(ResponseJson(ApiResponse::success(device_start_response)))
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(use_ts_enum)]
pub enum DevicePollStatus {
    SlowDown,
    AuthorizationPending,
    Success,
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(use_ts_enum)]
pub enum CheckTokenResponse {
    Valid,
    Invalid,
}

#[derive(Serialize, ts_rs::TS)]
pub struct ClaudeSessionResponse {
    #[ts(type = "string")]
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
pub struct ClaudeSessionInput {
    pub input: String,
}

/// POST /auth/github/device/poll
async fn device_poll(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<DevicePollStatus>>, ApiError> {
    let user_info = match deployment.auth().device_poll().await {
        Ok(info) => info,
        Err(AuthError::Pending(Continue::SlowDown)) => {
            return Ok(ResponseJson(ApiResponse::success(
                DevicePollStatus::SlowDown,
            )));
        }
        Err(AuthError::Pending(Continue::AuthorizationPending)) => {
            return Ok(ResponseJson(ApiResponse::success(
                DevicePollStatus::AuthorizationPending,
            )));
        }
        Err(e) => return Err(e.into()),
    };
    // Save to config
    {
        let config_path = utils::assets::config_path();
        let mut config = deployment.config().write().await;
        config.github.username = Some(user_info.username.clone());
        config.github.primary_email = user_info.primary_email.clone();
        if let Err(err) = deployment
            .secret_store()
            .put_secret(
                deployment.user_id(),
                SECRET_GITHUB_OAUTH,
                user_info.token.as_bytes(),
            )
            .await
        {
            tracing::error!("Failed to persist GitHub OAuth token: {err}");
            return Err(ApiError::Deployment(err.into()));
        }
        config.github.oauth_token = None;
        config.github_login_acknowledged = true; // Also acknowledge the GitHub login step
        save_config_to_file(&config.clone(), &config_path).await?;
    }

    // Claim orphaned data for first-time login
    // Get GitHub user ID and claim all data with user_id IS NULL
    {
        let gh = GitHubService::new(&user_info.token)?;
        match deployment
            .github_user_cache()
            .get_or_fetch(&user_info.token, &gh)
            .await
        {
            Ok(github_user) => {
                let user_id = format!("github_{}", github_user.id);
                tracing::info!("Claiming orphaned data for user: {}", user_id);

                // Claim projects
                match Project::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned projects", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "projects",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned projects to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned projects: {}", e);
                    }
                }

                // Claim tasks
                match Task::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned tasks", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "tasks",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned tasks to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned tasks: {}", e);
                    }
                }

                // Claim task attempts
                match TaskAttempt::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned task attempts", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "task_attempts",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned task attempts to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned task attempts: {}", e);
                    }
                }

                // Claim tags
                match Tag::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned tags", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "tags",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned tags to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned tags: {}", e);
                    }
                }

                // Claim images
                match Image::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned images", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "images",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned images to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned images: {}", e);
                    }
                }

                // Claim drafts
                match Draft::claim_orphaned(&deployment.db().pool, &user_id).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Claimed {} orphaned drafts", count);
                        deployment
                            .track_if_analytics_allowed(
                                "orphaned_data_claimed",
                                serde_json::json!({
                                    "type": "drafts",
                                    "count": count,
                                }),
                            )
                            .await;
                    }
                    Ok(_) => {
                        tracing::debug!("No orphaned drafts to claim");
                    }
                    Err(e) => {
                        tracing::error!("Failed to claim orphaned drafts: {}", e);
                    }
                }
            }
            Err(e) => {
                // Log error but don't fail the entire login process
                tracing::error!("Failed to fetch GitHub user for orphaned data claim: {}", e);
            }
        }
    }

    let _ = deployment.update_sentry_scope().await;
    let props = serde_json::json!({
        "username": user_info.username,
        "email": user_info.primary_email,
    });
    deployment
        .track_if_analytics_allowed("$identify", props)
        .await;
    Ok(ResponseJson(ApiResponse::success(
        DevicePollStatus::Success,
    )))
}

/// GET /auth/github/check
async fn github_check_token(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<CheckTokenResponse>>, ApiError> {
    let github_token = match deployment.github_token().await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Ok(ResponseJson(ApiResponse::success(
                CheckTokenResponse::Invalid,
            )));
        }
        Err(err) => return Err(ApiError::Deployment(DeploymentError::from(err))),
    };
    let gh = GitHubService::new(&github_token)?;
    match gh.check_token().await {
        Ok(()) => Ok(ResponseJson(ApiResponse::success(
            CheckTokenResponse::Valid,
        ))),
        Err(GitHubServiceError::TokenInvalid) => Ok(ResponseJson(ApiResponse::success(
            CheckTokenResponse::Invalid,
        ))),
        Err(e) => Err(e.into()),
    }
}

async fn claude_session_start(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<ClaudeSessionResponse>>, ApiError> {
    let session_id = deployment
        .claude_auth()
        .start_session()
        .await
        .map_err(|err| ApiError::Deployment(err.into()))?;
    Ok(ResponseJson(ApiResponse::success(ClaudeSessionResponse { session_id })))
}

async fn claude_session_stream(
    Path(session_id): Path<Uuid>,
    State(deployment): State<DeploymentImpl>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, ApiError>>>, ApiError> {
    let stream = deployment
        .claude_auth()
        .subscribe(&session_id)
        .map_err(|err| ApiError::Deployment(err.into()))?;

    let mapped = stream.filter_map(|item| async move {
        match item {
            Ok(payload) => match Event::default().json_data(&payload) {
                Ok(event) => Some(Ok(event)),
                Err(err) => Some(Err(ApiError::Deployment(
                    DeploymentError::Other(err.into()),
                ))),
            },
            Err(_) => None,
        }
    });

    Ok(Sse::new(mapped).keep_alive(KeepAlive::default()))
}

async fn claude_session_input(
    Path(session_id): Path<Uuid>,
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<ClaudeSessionInput>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    deployment
        .claude_auth()
        .send_input(&session_id, payload.input)
        .await
        .map_err(|err| ApiError::Deployment(err.into()))?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn claude_session_cancel(
    Path(session_id): Path<Uuid>,
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    deployment
        .claude_auth()
        .cancel_session(&session_id)
        .await
        .map_err(|err| ApiError::Deployment(err.into()))?;
    Ok(ResponseJson(ApiResponse::success(())))
}

async fn claude_logout(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    deployment
        .secret_store()
        .delete_secret(deployment.user_id(), SECRET_CLAUDE_ACCESS)
        .await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

/// Middleware to set Sentry user context for every request
pub async fn sentry_user_context_middleware(
    State(deployment): State<DeploymentImpl>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let _ = deployment.update_sentry_scope().await;
    Ok(next.run(req).await)
}
