/// Authentication module for multi-user support
///
/// This module provides the AuthenticatedUser type which is injected
/// into API handlers via Axum's Extension mechanism.

use serde::{Deserialize, Serialize};

/// Represents an authenticated user from GitHub OAuth
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// Internal user ID (format: "github_{github_id}")
    pub user_id: String,

    /// GitHub user ID (numeric)
    pub github_id: i64,

    /// GitHub username (login)
    pub username: String,

    /// GitHub avatar URL
    pub avatar_url: Option<String>,
}

impl AuthenticatedUser {
    /// Create from GitHub user data
    pub fn from_github_user(github_id: i64, username: String, avatar_url: Option<String>) -> Self {
        Self {
            user_id: format!("github_{}", github_id),
            github_id,
            username,
            avatar_url,
        }
    }
}
