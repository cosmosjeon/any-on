use std::{num::NonZeroUsize, sync::Arc};

/// GitHub User Cache for reducing API calls
///
/// Uses LRU cache with 5-minute TTL to cache GitHub user information
/// Retrieved via GitHub token validation.
use chrono::{DateTime, TimeDelta, Utc};
use lru::LruCache;
use tokio::sync::RwLock;

use super::github_service::{GitHubService, GitHubServiceError};

/// Cached GitHub user information
#[derive(Clone, Debug)]
struct CachedUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
    cached_at: DateTime<Utc>,
}

/// GitHub User with basic info
#[derive(Clone, Debug)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
}

/// Thread-safe LRU cache for GitHub user data
#[derive(Clone)]
pub struct GitHubUserCache {
    cache: Arc<RwLock<LruCache<String, CachedUser>>>,
    ttl_seconds: i64,
}

impl GitHubUserCache {
    /// Create a new cache with default settings
    /// - Capacity: 1000 users
    /// - TTL: 5 minutes
    pub fn new() -> Self {
        Self::with_capacity_and_ttl(1000, 300)
    }

    /// Create a cache with custom capacity and TTL
    pub fn with_capacity_and_ttl(capacity: usize, ttl_seconds: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
            ttl_seconds,
        }
    }

    /// Get user info from cache or fetch from GitHub API
    pub async fn get_or_fetch(
        &self,
        token: &str,
        gh: &GitHubService,
    ) -> Result<GitHubUser, GitHubServiceError> {
        let cache_key = self.compute_cache_key(token);

        // Check cache first
        {
            let mut guard = self.cache.write().await;
            if let Some(cached) = guard.get(&cache_key) {
                let age = Utc::now() - cached.cached_at;
                if age.num_seconds() < self.ttl_seconds {
                    tracing::debug!(
                        "GitHub user cache hit for user {} (age: {}s)",
                        cached.login,
                        age.num_seconds()
                    );
                    return Ok(GitHubUser {
                        id: cached.id,
                        login: cached.login.clone(),
                        avatar_url: cached.avatar_url.clone(),
                    });
                } else {
                    // Expired, remove it
                    tracing::debug!(
                        "GitHub user cache expired for user {} (age: {}s)",
                        cached.login,
                        age.num_seconds()
                    );
                    guard.pop(&cache_key);
                }
            }
        }

        // Cache miss or expired - fetch from GitHub
        tracing::debug!("GitHub user cache miss, fetching from API");
        let user = gh.get_current_user().await?;

        // Update cache
        {
            let mut guard = self.cache.write().await;
            guard.put(
                cache_key,
                CachedUser {
                    id: user.id,
                    login: user.login.clone(),
                    avatar_url: user.avatar_url.clone(),
                    cached_at: Utc::now(),
                },
            );
        }

        Ok(user)
    }

    /// Invalidate cache for a specific token
    pub async fn invalidate(&self, token: &str) {
        let cache_key = self.compute_cache_key(token);
        let mut guard = self.cache.write().await;
        guard.pop(&cache_key);
        tracing::debug!("Invalidated GitHub user cache");
    }

    /// Clear all cached data
    pub async fn clear(&self) {
        let mut guard = self.cache.write().await;
        guard.clear();
        tracing::info!("Cleared GitHub user cache");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let guard = self.cache.read().await;
        CacheStats {
            size: guard.len(),
            capacity: guard.cap().get(),
        }
    }

    /// Compute MD5 hash of token for cache key
    fn compute_cache_key(&self, token: &str) -> String {
        format!("{:x}", md5::compute(token))
    }
}

impl Default for GitHubUserCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_consistency() {
        let cache = GitHubUserCache::new();
        let token = "test_token_123";

        let key1 = cache.compute_cache_key(token);
        let key2 = cache.compute_cache_key(token);

        assert_eq!(key1, key2, "Cache keys should be consistent");
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = GitHubUserCache::with_capacity_and_ttl(100, 300);
        let stats = cache.stats().await;

        assert_eq!(stats.size, 0);
        assert_eq!(stats.capacity, 100);
    }

    #[tokio::test]
    async fn test_invalidate() {
        let cache = GitHubUserCache::new();
        let token = "test_token";

        // Manually insert into cache
        {
            let mut guard = cache.cache.write().await;
            guard.put(
                cache.compute_cache_key(token),
                CachedUser {
                    id: 123,
                    login: "testuser".to_string(),
                    avatar_url: None,
                    cached_at: Utc::now(),
                },
            );
        }

        let stats = cache.stats().await;
        assert_eq!(stats.size, 1);

        // Invalidate
        cache.invalidate(token).await;

        let stats = cache.stats().await;
        assert_eq!(stats.size, 0);
    }
}
