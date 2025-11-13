use axum::{
    Json, Router,
    extract::{
        Path, Request, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{
        Json as ResponseJson, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use db::models::{
    draft::Draft, image::Image, project::Project, tag::Tag, task::Task, task_attempt::TaskAttempt,
};
use deployment::{Deployment, DeploymentError};
use futures_util::{sink::SinkExt, stream::StreamExt};
use octocrab::auth::Continue;
use serde::{Deserialize, Serialize};
use services::services::{
    auth::{AuthError, DeviceFlowStartResponse},
    claude_auth_pty::ClaudePtyLogEntry,
    config::save_config_to_file,
    github_service::{GitHubService, GitHubServiceError},
    secret_store::{SECRET_CLAUDE_ACCESS, SECRET_GITHUB_OAUTH},
};
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[cfg(not(feature = "cloud"))]
pub const DEV_LOGIN_PLACEHOLDER_TOKEN: &str = "dev-login-placeholder-token";

/// POST /auth/dev-login (local builds only)
/// Logs in with a development user without GitHub OAuth
#[cfg(not(feature = "cloud"))]
async fn dev_login(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<DevicePollStatus>>, ApiError> {
    tracing::warn!("🔧 DEV MODE: Logging in with development user");

    // Use the current deployment's user_id (hardware-based) instead of a fixed "dev_user"
    let user_id = deployment.user_id();

    // Save to config (same as device_poll)
    {
        let config_path = utils::assets::config_path();
        let mut config = deployment.config().write().await;
        config.github.username = Some("Dev User".to_string());
        config.github.primary_email = Some("dev@localhost".to_string());
        config.github.oauth_token = None;
        config.github_login_acknowledged = true;
        save_config_to_file(&config.clone(), &config_path).await?;
    }

    // Mark GitHub as connected by storing a placeholder OAuth secret
    deployment
        .secret_store()
        .put_secret(
            deployment.user_id(),
            SECRET_GITHUB_OAUTH,
            DEV_LOGIN_PLACEHOLDER_TOKEN.as_bytes(),
        )
        .await?;

    // Claim orphaned data for dev user
    {
        tracing::info!("Claiming orphaned data for dev user: {}", user_id);

        // Claim projects
        match Project::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned projects", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned projects to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned projects: {}", e);
            }
        }

        // Claim tasks
        match Task::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned tasks", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned tasks to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned tasks: {}", e);
            }
        }

        // Claim task attempts
        match TaskAttempt::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned task attempts", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned task attempts to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned task attempts: {}", e);
            }
        }

        // Claim tags
        match Tag::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned tags", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned tags to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned tags: {}", e);
            }
        }

        // Claim images
        match Image::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned images", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned images to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned images: {}", e);
            }
        }

        // Claim drafts
        match Draft::claim_orphaned(&deployment.db().pool, user_id).await {
            Ok(count) if count > 0 => {
                tracing::info!("Claimed {} orphaned drafts", count);
            }
            Ok(_) => {
                tracing::debug!("No orphaned drafts to claim");
            }
            Err(e) => {
                tracing::error!("Failed to claim orphaned drafts: {}", e);
            }
        }
    }

    let _ = deployment.update_sentry_scope().await;

    tracing::info!("✅ DEV MODE: Login successful");
    Ok(ResponseJson(ApiResponse::success(
        DevicePollStatus::Success,
    )))
}

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    let mut router = Router::new()
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
        .route("/auth/claude/pty", get(claude_pty_websocket))
        .route(
            "/auth/claude/pty/session/{session_id}/log",
            get(claude_pty_session_log),
        )
        .layer(from_fn_with_state(
            deployment.clone(),
            sentry_user_context_middleware,
        ));

    // Add dev login endpoint only for local (non-cloud) builds
    #[cfg(not(feature = "cloud"))]
    {
        router = router.route("/auth/dev-login", post(dev_login));
    }

    router
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

#[derive(Serialize)]
pub struct ClaudePtySessionLogResponse {
    #[serde(rename = "session_id")]
    pub session_id: Uuid,
    pub entries: Vec<ClaudePtyLogEntry>,
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
    tracing::info!("Received Claude session start request");
    let session_id = deployment
        .claude_auth()
        .start_session()
        .await
        .map_err(|err| {
            tracing::error!("Failed to start Claude session: {}", err);
            ApiError::Deployment(err.into())
        })?;
    tracing::info!(session_id = %session_id, "Claude session started successfully");
    Ok(ResponseJson(ApiResponse::success(ClaudeSessionResponse {
        session_id,
    })))
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
                Err(err) => Some(Err(ApiError::Deployment(DeploymentError::Other(
                    err.into(),
                )))),
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

/// WebSocket endpoint for PTY-based Claude login
async fn claude_pty_websocket(
    ws: WebSocketUpgrade,
    State(deployment): State<DeploymentImpl>,
) -> Response {
    ws.on_upgrade(move |socket| handle_claude_pty(socket, deployment))
}

async fn claude_pty_session_log(
    Path(session_id): Path<Uuid>,
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<ClaudePtySessionLogResponse>>, ApiError> {
    let entries = deployment
        .claude_auth_pty()
        .get_logs(&session_id)
        .await
        .map_err(|err| ApiError::Deployment(DeploymentError::Other(err.into())))?;

    let response = ClaudePtySessionLogResponse {
        session_id,
        entries,
    };

    Ok(ResponseJson(ApiResponse::success(response)))
}

async fn handle_claude_pty(mut socket: WebSocket, deployment: DeploymentImpl) {
    // Start PTY session
    let session_id = match deployment
        .claude_auth_pty()
        .start_session(deployment.user_id())
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "Failed to start PTY session");
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    format!("Error: {}", e).into(),
                ))
                .await;
            return;
        }
    };

    tracing::info!(session_id = %session_id, "PTY session started");

    // Spawn task to read from PTY and send to WebSocket
    let deployment_clone = deployment.clone();
    let deployment_check = deployment.clone();
    let user_id = deployment.user_id().to_string();
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (success_tx, mut success_rx) = tokio::sync::mpsc::channel::<()>(1);

    let read_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        let meta_message = format!("__CLAUDE_META__{{\"sessionId\":\"{}\"}}", session_id);
        tracing::debug!(
            session_id = %session_id,
            "📤 Sending session metadata to frontend"
        );
        let _ = ws_sender
            .send(axum::extract::ws::Message::Text(meta_message.into()))
            .await;
        loop {
            tokio::select! {
                result = deployment_clone.claude_auth_pty().read_output(&session_id, &mut buf) => {
                    match result {
                        Ok(0) => {
                            tracing::debug!(
                                session_id = %session_id,
                                "📭 PTY output EOF reached"
                            );
                            break;
                        }
                        Ok(n) => {
                            tracing::trace!(
                                session_id = %session_id,
                                bytes = n,
                                "📨 Sending PTY output to WebSocket"
                            );
                            if ws_sender
                                .send(axum::extract::ws::Message::Binary(buf[..n].to_vec().into()))
                                .await
                                .is_err()
                            {
                                tracing::error!(
                                    session_id = %session_id,
                                    "❌ WebSocket send failed, closing read task"
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                session_id = %session_id,
                                error = %e,
                                "❌ Failed to read from PTY"
                            );
                            break;
                        }
                    }
                }
                _ = success_rx.recv() => {
                    tracing::info!(
                        session_id = %session_id,
                        "🎉 Success signal received from check task!"
                    );
                    // Send success message to frontend
                    let success_msg = "\r\n\r\n✅ 로그인 성공! Credential이 저장되었습니다.\r\n";
                    tracing::info!(
                        session_id = %session_id,
                        message = success_msg,
                        "📤 Sending success message to frontend"
                    );
                    let send_result = ws_sender.send(axum::extract::ws::Message::Text(success_msg.into())).await;
                    if send_result.is_ok() {
                        tracing::info!(
                            session_id = %session_id,
                            "✅ Success message sent to frontend successfully"
                        );
                    } else {
                        tracing::error!(
                            session_id = %session_id,
                            "❌ Failed to send success message to frontend"
                        );
                    }
                    break;
                }
            }
        }
        tracing::debug!(
            session_id = %session_id,
            "🛑 Read task exiting"
        );
    });

    // Spawn task to check for login success
    let check_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        let mut check_count = 0;
        tracing::info!(
            session_id = %session_id,
            "🔄 Starting login success check task (polling every 2s)"
        );
        loop {
            interval.tick().await;
            check_count += 1;
            tracing::debug!(
                session_id = %session_id,
                check_count = check_count,
                "⏱️  Performing login success check #{}", check_count
            );
            match deployment_check
                .claude_auth_pty()
                .check_login_success(&session_id, &user_id)
                .await
            {
                Ok(true) => {
                    tracing::info!(
                        session_id = %session_id,
                        check_count = check_count,
                        "🎊 Login success detected! Sending signal to read task..."
                    );
                    let send_result = success_tx.send(()).await;
                    if send_result.is_ok() {
                        tracing::info!(
                            session_id = %session_id,
                            "✅ Success signal sent to read task successfully"
                        );
                    } else {
                        tracing::error!(
                            session_id = %session_id,
                            "❌ Failed to send success signal to read task"
                        );
                    }
                    break;
                }
                Ok(false) => {
                    tracing::trace!(
                        session_id = %session_id,
                        check_count = check_count,
                        "⏭️  No credential found yet, continuing to check..."
                    );
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %session_id,
                        error = %e,
                        "❌ Failed to check login success, stopping check task"
                    );
                    break;
                }
            }
        }
        tracing::debug!(
            session_id = %session_id,
            "🛑 Check task exiting"
        );
    });

    // Read from WebSocket and write to PTY
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Binary(data)) => {
                if let Err(e) = deployment
                    .claude_auth_pty()
                    .write_input(&session_id, &data)
                    .await
                {
                    tracing::error!(error = %e, "Failed to write to PTY");
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Text(data)) => {
                let data_bytes = data.as_bytes();
                if let Err(e) = deployment
                    .claude_auth_pty()
                    .write_input(&session_id, data_bytes)
                    .await
                {
                    tracing::error!(error = %e, "Failed to write to PTY");
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => break,
            Err(e) => {
                tracing::error!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    let _ = deployment
        .claude_auth_pty()
        .cancel_session(&session_id)
        .await;
    read_task.abort();
    check_task.abort();
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
