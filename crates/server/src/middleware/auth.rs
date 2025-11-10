//! Authentication middleware for multi-user support
//!
//! This middleware checks if the user has a valid GitHub token and injects
//! the authenticated user information into the request.

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use services::services::{
    github_service::{GitHubService, GitHubServiceError},
    secret_store::SECRET_GITHUB_OAUTH,
};

use crate::{auth::AuthenticatedUser, DeploymentImpl};

/// Middleware that requires GitHub authentication
///
/// This middleware:
/// 1. Retrieves the GitHub token from SecretStore
/// 2. Validates the token with GitHub API (using cache)
/// 3. Injects AuthenticatedUser into the request extensions
/// 4. Returns 401 if authentication fails
///
/// # Example
/// ```rust
/// Router::new()
///     .route("/projects", get(get_projects))
///     .layer(from_fn_with_state(deployment.clone(), require_auth))
/// ```
pub async fn require_auth<B>(
    State(deployment): State<DeploymentImpl>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Step 1: Get GitHub token from SecretStore
    // "이 사용자가 GitHub 로그인 했을 때 받은 토큰을 가져옴"
    let token = deployment
        .secret_store()
        .get_secret_string(deployment.user_id(), SECRET_GITHUB_OAUTH)
        .await
        .map_err(|err| {
            tracing::warn!("Failed to retrieve GitHub token: {}", err);
            StatusCode::UNAUTHORIZED
        })?
        .ok_or_else(|| {
            tracing::debug!("No GitHub token found for user");
            StatusCode::UNAUTHORIZED
        })?;

    // Step 2: Create GitHub service client
    let gh = GitHubService::new(&token).map_err(|err| {
        tracing::error!("Failed to create GitHub service: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Step 3: Validate token and get user info (with caching)
    // "토큰이 유효한지 확인하고, 사용자 정보 가져옴"
    // "캐시가 있으면 빠르게, 없으면 GitHub API 호출"
    let user = match deployment
        .github_user_cache()
        .get_or_fetch(&token, &gh)
        .await
    {
        Ok(user) => user,
        Err(GitHubServiceError::TokenInvalid) => {
            // Token is expired or invalid
            // "토큰이 만료됨 → 삭제하고 재로그인 유도"
            tracing::info!("GitHub token expired or invalid, deleting from store");

            // Delete expired token from SecretStore
            let _ = deployment
                .secret_store()
                .delete_secret(deployment.user_id(), SECRET_GITHUB_OAUTH)
                .await;

            // Invalidate cache
            deployment.github_user_cache().invalidate(&token).await;

            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            // Other GitHub API errors
            tracing::error!("GitHub API error: {}", err);
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    // Step 4: Create AuthenticatedUser and inject into request
    // "검증된 사용자 정보를 Request에 추가 → 핸들러에서 사용 가능"
    let authenticated_user = AuthenticatedUser::from_github_user(
        user.id,
        user.login,
        user.avatar_url,
    );

    tracing::debug!(
        "Authenticated user: {} ({})",
        authenticated_user.username,
        authenticated_user.user_id
    );

    req.extensions_mut().insert(authenticated_user);

    // Step 5: Continue to the next handler
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_creation() {
        let user = AuthenticatedUser::from_github_user(
            123456,
            "octocat".to_string(),
            Some("https://avatar.url".to_string()),
        );

        assert_eq!(user.user_id, "github_123456");
        assert_eq!(user.github_id, 123456);
        assert_eq!(user.username, "octocat");
    }
}
