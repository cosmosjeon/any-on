#![cfg(feature = "cloud")]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use bollard::models::HostConfig;
use dashmap::DashMap;
use db::{
    DBService,
    models::{
        execution_process::{ExecutionContext, ExecutionProcess, ExecutionProcessStatus},
        task_attempt::TaskAttempt,
    },
};
use executors::actions::ExecutorAction;
use tokio::sync::RwLock;
use utils::{log_msg::LogMsg, msg_store::MsgStore};
use uuid::Uuid;

use crate::services::{
    container::{ContainerError, ContainerRef, ContainerService},
    docker_poc::DockerHarness,
    git::GitService,
};

/// Default image used when no explicit image is provided via configuration.
const DEFAULT_IMAGE: &str = "anyon-claude:latest";
const DEFAULT_MOUNT: &str = "/workspace";

#[derive(Debug, Clone)]
pub struct CloudContainerSettings {
    pub default_image: String,
    pub workspace_mount: String,
    pub idle_command: Vec<String>,
}

impl Default for CloudContainerSettings {
    fn default() -> Self {
        Self {
            default_image: DEFAULT_IMAGE.to_string(),
            workspace_mount: DEFAULT_MOUNT.to_string(),
            idle_command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "while true; do sleep 60; done".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
struct ProvisionedContainer {
    container_id: String,
    worktree: PathBuf,
}

#[derive(Clone)]
pub struct CloudContainerService<T>
where
    T: ContainerService + Clone + Send + Sync,
{
    inner: T,
    docker: Arc<DockerHarness>,
    settings: Arc<CloudContainerSettings>,
    provisioned: Arc<DashMap<Uuid, ProvisionedContainer>>,
    provision_lock: Arc<tokio::sync::Mutex<()>>,
}

impl<T> CloudContainerService<T>
where
    T: ContainerService + Clone + Send + Sync,
{
    pub async fn new(inner: T, settings: CloudContainerSettings) -> Result<Self, ContainerError> {
        let harness = DockerHarness::connect()
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        Ok(Self {
            inner,
            docker: Arc::new(harness),
            settings: Arc::new(settings),
            provisioned: Arc::new(DashMap::new()),
            provision_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    async fn ensure_runner(
        &self,
        task_attempt: &TaskAttempt,
        worktree_path: &Path,
    ) -> Result<String, ContainerError> {
        let normalized = Self::normalize_path(worktree_path);

        if let Some(entry) = self.provisioned.get(&task_attempt.id) {
            let container_id = entry.container_id.clone();
            let recorded_worktree = entry.worktree.clone();
            drop(entry);

            if recorded_worktree == normalized
                && self.docker.inspect_container(&container_id).await.is_ok()
            {
                return Ok(container_id);
            }
        }

        let _guard = self.provision_lock.lock().await;

        if let Some(entry) = self.provisioned.get(&task_attempt.id) {
            let container_id = entry.container_id.clone();
            let recorded_worktree = entry.worktree.clone();
            drop(entry);

            if recorded_worktree == normalized
                && self.docker.inspect_container(&container_id).await.is_ok()
            {
                return Ok(container_id);
            } else {
                self.provisioned.remove(&task_attempt.id);
            }
        }

        self.docker
            .ensure_image(&self.settings.default_image)
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        let host_config = HostConfig {
            binds: Some(vec![format!(
                "{}:{}:rw",
                normalized.display(),
                self.settings.workspace_mount
            )]),
            ..Default::default()
        };

        let env = vec![
            format!("ANYON_WORKSPACE={}", self.settings.workspace_mount),
            format!("ANYON_ATTEMPT_ID={}", task_attempt.id),
        ];

        let container_id = self
            .docker
            .create_container(
                &format!("task-attempt-{}", task_attempt.id),
                &self.settings.default_image,
                Some(self.settings.idle_command.clone()),
                Some(host_config),
                Some(env),
                true,
            )
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        self.docker
            .start_container(&container_id)
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        self.provisioned.insert(
            task_attempt.id,
            ProvisionedContainer {
                container_id: container_id.clone(),
                worktree: normalized,
            },
        );

        Ok(container_id)
    }

    async fn teardown(&self, attempt_id: &Uuid) {
        if let Some((_, record)) = self.provisioned.remove(attempt_id) {
            if let Err(err) = self
                .docker
                .stop_container_with_timeout(&record.container_id, Duration::from_secs(2))
                .await
            {
                tracing::warn!(
                    "failed to stop cloud container {}: {err}",
                    record.container_id
                );
            }
            if let Err(err) = self
                .docker
                .remove_container(&record.container_id, true)
                .await
            {
                tracing::warn!(
                    "failed to remove cloud container {}: {err}",
                    record.container_id
                );
            }
        }
    }

    fn normalize_path(path: &Path) -> PathBuf {
        dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}

#[async_trait]
impl<T> ContainerService for CloudContainerService<T>
where
    T: ContainerService + Clone + Send + Sync,
{
    fn msg_stores(&self) -> &Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>> {
        self.inner.msg_stores()
    }

    fn db(&self) -> &DBService {
        self.inner.db()
    }

    fn git(&self) -> &GitService {
        self.inner.git()
    }

    fn task_attempt_to_current_dir(&self, task_attempt: &TaskAttempt) -> PathBuf {
        self.inner.task_attempt_to_current_dir(task_attempt)
    }

    async fn create(&self, task_attempt: &TaskAttempt) -> Result<ContainerRef, ContainerError> {
        let container_ref = self.inner.create(task_attempt).await?;
        let worktree_path = PathBuf::from(&container_ref);
        self.ensure_runner(task_attempt, &worktree_path).await?;
        Ok(container_ref)
    }

    async fn delete_inner(&self, task_attempt: &TaskAttempt) -> Result<(), ContainerError> {
        self.teardown(&task_attempt.id).await;
        self.inner.delete_inner(task_attempt).await
    }

    async fn ensure_container_exists(
        &self,
        task_attempt: &TaskAttempt,
    ) -> Result<ContainerRef, ContainerError> {
        let container_ref = self.inner.ensure_container_exists(task_attempt).await?;
        let worktree_path = PathBuf::from(&container_ref);
        self.ensure_runner(task_attempt, &worktree_path).await?;
        Ok(container_ref)
    }

    async fn is_container_clean(&self, task_attempt: &TaskAttempt) -> Result<bool, ContainerError> {
        self.inner.is_container_clean(task_attempt).await
    }

    async fn start_execution_inner(
        &self,
        task_attempt: &TaskAttempt,
        execution_process: &ExecutionProcess,
        executor_action: &ExecutorAction,
    ) -> Result<(), ContainerError> {
        if let Some(container_ref) = &task_attempt.container_ref {
            let worktree_path = PathBuf::from(container_ref);
            self.ensure_runner(task_attempt, &worktree_path).await?;
        }
        self.inner
            .start_execution_inner(task_attempt, execution_process, executor_action)
            .await
    }

    async fn stop_execution(
        &self,
        execution_process: &ExecutionProcess,
        status: ExecutionProcessStatus,
    ) -> Result<(), ContainerError> {
        self.inner.stop_execution(execution_process, status).await
    }

    async fn try_commit_changes(&self, ctx: &ExecutionContext) -> Result<bool, ContainerError> {
        self.inner.try_commit_changes(ctx).await
    }

    async fn copy_project_files(
        &self,
        source_dir: &Path,
        target_dir: &Path,
        copy_files: &str,
    ) -> Result<(), ContainerError> {
        self.inner
            .copy_project_files(source_dir, target_dir, copy_files)
            .await
    }

    async fn stream_diff(
        &self,
        task_attempt: &TaskAttempt,
        stats_only: bool,
    ) -> Result<futures::stream::BoxStream<'static, Result<LogMsg, std::io::Error>>, ContainerError>
    {
        self.inner.stream_diff(task_attempt, stats_only).await
    }

    async fn git_branch_prefix(&self) -> String {
        self.inner.git_branch_prefix().await
    }
}
