use std::time::Duration;

use db::{
    DBService,
    models::{
        merge::{Merge, MergeStatus, PrMerge},
        task::{Task, TaskStatus},
        task_attempt::{TaskAttempt, TaskAttemptError},
    },
};
use serde_json::json;
use sqlx::error::Error as SqlxError;
use thiserror::Error;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::services::{
    analytics::AnalyticsContext,
    github_service::{GitHubRepoInfo, GitHubService, GitHubServiceError},
    secret_store::{SECRET_GITHUB_OAUTH, SECRET_GITHUB_PAT, SecretStore, SecretStoreError},
};

#[derive(Debug, Error)]
enum PrMonitorError {
    #[error("No GitHub token configured")]
    NoGitHubToken,
    #[error(transparent)]
    GitHubServiceError(#[from] GitHubServiceError),
    #[error(transparent)]
    TaskAttemptError(#[from] TaskAttemptError),
    #[error(transparent)]
    Sqlx(#[from] SqlxError),
    #[error(transparent)]
    SecretStore(#[from] SecretStoreError),
}

/// Service to monitor GitHub PRs and update task status when they are merged
pub struct PrMonitorService {
    db: DBService,
    poll_interval: Duration,
    analytics: Option<AnalyticsContext>,
    secret_store: SecretStore,
    user_id: String,
}

impl PrMonitorService {
    pub async fn spawn(
        db: DBService,
        analytics: Option<AnalyticsContext>,
        secret_store: SecretStore,
        user_id: String,
    ) -> tokio::task::JoinHandle<()> {
        let service = Self {
            db,
            poll_interval: Duration::from_secs(60), // Check every minute
            analytics,
            secret_store,
            user_id,
        };
        tokio::spawn(async move {
            service.start().await;
        })
    }

    async fn start(&self) {
        info!(
            "Starting PR monitoring service with interval {:?}",
            self.poll_interval
        );

        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.check_all_open_prs().await {
                error!("Error checking open PRs: {}", e);
            }
        }
    }

    /// Check all open PRs for updates with the provided GitHub token
    async fn check_all_open_prs(&self) -> Result<(), PrMonitorError> {
        let open_prs = Merge::get_open_prs(&self.db.pool).await?;

        if open_prs.is_empty() {
            debug!("No open PRs to check");
            return Ok(());
        }

        info!("Checking {} open PRs", open_prs.len());

        for pr_merge in open_prs {
            match self.check_pr_status(&pr_merge).await {
                Err(PrMonitorError::NoGitHubToken) => {
                    warn!("No GitHub token configured, cannot check PR status");
                }
                Err(e) => {
                    error!(
                        "Error checking PR #{} for attempt {}: {}",
                        pr_merge.pr_info.number, pr_merge.task_attempt_id, e
                    );
                }
                Ok(_) => {}
            }
        }
        Ok(())
    }

    /// Check the status of a specific PR
    async fn check_pr_status(&self, pr_merge: &PrMerge) -> Result<(), PrMonitorError> {
        let github_token = self.fetch_github_token().await?;

        let github_service = GitHubService::new(&github_token)?;

        let repo_info = GitHubRepoInfo::from_remote_url(&pr_merge.pr_info.url)?;

        let pr_status = github_service
            .update_pr_status(&repo_info, pr_merge.pr_info.number)
            .await?;

        debug!(
            "PR #{} status: {:?} (was open)",
            pr_merge.pr_info.number, pr_status.status
        );

        // Update the PR status in the database
        if !matches!(&pr_status.status, MergeStatus::Open) {
            // Update merge status with the latest information from GitHub
            Merge::update_status(
                &self.db.pool,
                pr_merge.id,
                pr_status.status.clone(),
                pr_status.merge_commit_sha,
            )
            .await?;

            // If the PR was merged, update the task status to done
            if matches!(&pr_status.status, MergeStatus::Merged)
                && let Some(task_attempt) =
                    TaskAttempt::find_by_id(&self.db.pool, pr_merge.task_attempt_id).await?
            {
                info!(
                    "PR #{} was merged, updating task {} to done",
                    pr_merge.pr_info.number, task_attempt.task_id
                );
                Task::update_status(&self.db.pool, task_attempt.task_id, TaskStatus::Done).await?;

                // Track analytics event
                if let Some(analytics) = &self.analytics
                    && let Ok(Some(task)) =
                        Task::find_by_id(&self.db.pool, task_attempt.task_id).await
                {
                    analytics.analytics_service.track_event(
                        &analytics.user_id,
                        "pr_merged",
                        Some(json!({
                            "task_id": task_attempt.task_id.to_string(),
                            "task_attempt_id": task_attempt.id.to_string(),
                            "project_id": task.project_id.to_string(),
                        })),
                    );
                }
            }
        }

        Ok(())
    }

    async fn fetch_github_token(&self) -> Result<String, PrMonitorError> {
        if let Some(pat) = self
            .secret_store
            .get_secret_string(&self.user_id, SECRET_GITHUB_PAT)
            .await?
        {
            return Ok(pat);
        }
        if let Some(token) = self
            .secret_store
            .get_secret_string(&self.user_id, SECRET_GITHUB_OAUTH)
            .await?
        {
            return Ok(token);
        }
        Err(PrMonitorError::NoGitHubToken)
    }
}
