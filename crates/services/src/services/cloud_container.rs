#![cfg(feature = "cloud")]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use bollard::models::HostConfig;
use command_group::{AsyncCommandGroup, AsyncGroupChild};
use dashmap::DashMap;
use db::{
    DBService,
    models::{
        execution_process::{ExecutionContext, ExecutionProcess, ExecutionProcessStatus},
        task_attempt::TaskAttempt,
    },
};
use executors::{
    actions::ExecutorAction,
    command::{CommandRuntime, ExecutionCommand, StdioConfig},
    executors::ExecutorError,
};
use tokio::{process::Command, sync::RwLock};
use utils::{log_msg::LogMsg, msg_store::MsgStore, path::get_anyon_temp_dir};
use uuid::Uuid;

use crate::services::{
    container::{ContainerError, ContainerRef, ContainerService},
    docker_poc::DockerHarness,
    git::GitService,
    secret_store::{SECRET_CLAUDE_ACCESS, SECRET_GITHUB_OAUTH, SECRET_GITHUB_PAT, SecretStore},
};

struct DockerCommandRuntime {
    container_id: String,
    host_worktree: PathBuf,
    workspace_mount: PathBuf,
    base_env: Vec<(String, String)>,
}

impl DockerCommandRuntime {
    fn new(
        container_id: String,
        host_worktree: PathBuf,
        workspace_mount: PathBuf,
        base_env: Vec<(String, String)>,
    ) -> Self {
        Self {
            container_id,
            host_worktree,
            workspace_mount,
            base_env,
        }
    }

    fn container_workdir(&self, current_dir: &Path) -> PathBuf {
        match current_dir.strip_prefix(&self.host_worktree) {
            Ok(rem) => self.workspace_mount.join(rem),
            Err(_) => self.workspace_mount.clone(),
        }
    }
}

#[derive(Clone, Copy)]
enum IoStream {
    Stdin,
    Stdout,
    Stderr,
}

fn apply_stdio(command: &mut Command, config: StdioConfig, stream: IoStream) {
    let stdio = match config {
        StdioConfig::Inherit => Stdio::inherit(),
        StdioConfig::Piped => Stdio::piped(),
        StdioConfig::Null => Stdio::null(),
    };

    match stream {
        IoStream::Stdin => {
            command.stdin(stdio);
        }
        IoStream::Stdout => {
            command.stdout(stdio);
        }
        IoStream::Stderr => {
            command.stderr(stdio);
        }
    }
}

async fn write_secret_file(path: &Path, data: &[u8]) -> Result<(), ContainerError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    tokio::fs::write(path, data)
        .await
        .map_err(ContainerError::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        tokio::fs::set_permissions(path, perms)
            .await
            .map_err(ContainerError::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn container_workdir_maps_to_workspace() {
        let runtime = DockerCommandRuntime::new(
            "cid".into(),
            PathBuf::from("/host/worktree"),
            PathBuf::from("/workspace"),
            vec![],
        );

        let inside = runtime.container_workdir(Path::new("/host/worktree/project"));
        assert_eq!(inside, PathBuf::from("/workspace/project"));

        let outside = runtime.container_workdir(Path::new("/somewhere/else"));
        assert_eq!(outside, PathBuf::from("/workspace"));
    }

    #[tokio::test]
    async fn write_secret_file_creates_and_sets_permissions() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("nested/secret.json");
        write_secret_file(&target, b"token").await.unwrap();

        assert!(target.exists());
        let content = tokio::fs::read(&target).await.unwrap();
        assert_eq!(content, b"token");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "secret files should be 600");
        }
    }
}

#[async_trait]
impl CommandRuntime for DockerCommandRuntime {
    async fn spawn(&self, command: ExecutionCommand) -> Result<AsyncGroupChild, ExecutorError> {
        let mut process = Command::new("docker");
        process.arg("exec").arg("-i");

        let workdir = self.container_workdir(command.current_dir_path());
        process
            .arg("--workdir")
            .arg(workdir.to_string_lossy().to_string());

        for (key, value) in &self.base_env {
            process.arg("--env").arg(format!("{}={}", key, value));
        }

        for (key, value) in command.env_vars() {
            process.arg("--env").arg(format!(
                "{}={}",
                key.to_string_lossy(),
                value.to_string_lossy()
            ));
        }

        process.arg(&self.container_id);
        process.arg(command.program());
        process.args(command.args_slice());

        apply_stdio(&mut process, command.stdin_config(), IoStream::Stdin);
        apply_stdio(&mut process, command.stdout_config(), IoStream::Stdout);
        apply_stdio(&mut process, command.stderr_config(), IoStream::Stderr);

        if command.should_kill_on_drop() {
            process.kill_on_drop(true);
        }

        let child = process.group_spawn()?;
        Ok(child)
    }
}

/// Default image used when no explicit image is provided via configuration.
const DEFAULT_IMAGE: &str = "anyon-claude:latest";
const DEFAULT_MOUNT: &str = "/workspace";

#[derive(Debug, Clone)]
pub struct CloudContainerSettings {
    pub default_image: String,
    pub workspace_mount: String,
    pub secrets_mount: String,
    pub idle_command: Vec<String>,
}

impl Default for CloudContainerSettings {
    fn default() -> Self {
        Self {
            default_image: DEFAULT_IMAGE.to_string(),
            workspace_mount: DEFAULT_MOUNT.to_string(),
            secrets_mount: "/tmp/anyon-secrets".to_string(),
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
    secret_dir: PathBuf,
}

#[derive(Clone)]
pub struct CloudContainerService<T>
where
    T: ContainerService + Clone + Send + Sync,
{
    inner: T,
    docker: Arc<DockerHarness>,
    settings: Arc<CloudContainerSettings>,
    secret_store: SecretStore,
    user_id: String,
    provisioned: Arc<DashMap<Uuid, ProvisionedContainer>>,
    provision_lock: Arc<tokio::sync::Mutex<()>>,
}

impl<T> CloudContainerService<T>
where
    T: ContainerService + Clone + Send + Sync,
{
    pub async fn new(
        inner: T,
        secret_store: SecretStore,
        user_id: String,
        settings: CloudContainerSettings,
    ) -> Result<Self, ContainerError> {
        let harness = DockerHarness::connect()
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        Ok(Self {
            inner,
            docker: Arc::new(harness),
            settings: Arc::new(settings),
            secret_store,
            user_id,
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

        let secret_dir = self.secret_host_dir(&task_attempt.id);
        fs::create_dir_all(&secret_dir)?;

        let host_config = HostConfig {
            binds: Some(vec![
                format!(
                    "{}:{}:rw",
                    normalized.display(),
                    self.settings.workspace_mount
                ),
                format!(
                    "{}:{}:rw",
                    secret_dir.display(),
                    self.settings.secrets_mount
                ),
            ]),
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
                secret_dir,
            },
        );

        Ok(container_id)
    }

    async fn prepare_env(
        &self,
        container: &ProvisionedContainer,
    ) -> Result<Vec<(String, String)>, ContainerError> {
        if !container.secret_dir.exists() {
            fs::create_dir_all(&container.secret_dir)?;
        }

        let mut env = vec![(
            "ANYON_SECRET_DIR".into(),
            self.settings.secrets_mount.clone(),
        )];

        if let Some(claude_blob) = self
            .secret_store
            .get_secret(&self.user_id, SECRET_CLAUDE_ACCESS)
            .await
            .map_err(|err| ContainerError::Other(err.into()))?
        {
            let path = container.secret_dir.join("claude-config.json");
            write_secret_file(&path, &claude_blob).await?;
            env.push((
                "CLAUDE_CONFIG_PATH".into(),
                self.secret_mount_path("claude-config.json"),
            ));
        }

        let github_pat = self
            .secret_store
            .get_secret_string(&self.user_id, SECRET_GITHUB_PAT)
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;
        let github_oauth = self
            .secret_store
            .get_secret_string(&self.user_id, SECRET_GITHUB_OAUTH)
            .await
            .map_err(|err| ContainerError::Other(err.into()))?;

        let github_token = github_pat.or(github_oauth);
        if let Some(token) = github_token {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                let creds_path = container.secret_dir.join("github-credentials");
                let creds_content = format!("https://x-access-token:{}@github.com\n", trimmed);
                write_secret_file(&creds_path, creds_content.as_bytes()).await?;

                let gitconfig_path = container.secret_dir.join("gitconfig");
                let helper_path = self.secret_mount_path("github-credentials");
                let gitconfig_content =
                    format!("[credential]\n\thelper = store --file={}\n", helper_path);
                write_secret_file(&gitconfig_path, gitconfig_content.as_bytes()).await?;

                env.push((
                    "GIT_CONFIG_GLOBAL".into(),
                    self.secret_mount_path("gitconfig"),
                ));
                env.push(("GITHUB_TOKEN".into(), trimmed.to_string()));
                env.push(("GH_TOKEN".into(), trimmed.to_string()));
            }
        }

        Ok(env)
    }

    fn secret_mount_path(&self, file: &str) -> String {
        let mount = self.settings.secrets_mount.trim_end_matches('/');
        format!("{}/{}", mount, file)
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

            if let Err(err) = tokio::fs::remove_dir_all(&record.secret_dir).await {
                tracing::debug!(
                    "failed to remove secret directory {}: {err}",
                    record.secret_dir.display()
                );
            }
        }
    }

    fn normalize_path(path: &Path) -> PathBuf {
        dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    fn secret_host_dir(&self, attempt_id: &Uuid) -> PathBuf {
        get_anyon_temp_dir()
            .join("cloud-secrets")
            .join(attempt_id.to_string())
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

    async fn start_execution_with_runtime(
        &self,
        task_attempt: &TaskAttempt,
        execution_process: &ExecutionProcess,
        executor_action: &ExecutorAction,
        runtime: &dyn CommandRuntime,
    ) -> Result<(), ContainerError> {
        self.inner
            .start_execution_with_runtime(task_attempt, execution_process, executor_action, runtime)
            .await
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
        let provision = self.provisioned.get(&task_attempt.id).ok_or_else(|| {
            ContainerError::Other(anyhow::anyhow!(
                "Cloud container missing for attempt {}",
                task_attempt.id
            ))
        })?;
        let container_info = provision.clone();
        drop(provision);

        let base_env = self.prepare_env(&container_info).await?;
        let runtime = DockerCommandRuntime::new(
            container_info.container_id.clone(),
            container_info.worktree.clone(),
            PathBuf::from(&self.settings.workspace_mount),
            base_env,
        );

        self.inner
            .start_execution_with_runtime(
                task_attempt,
                execution_process,
                executor_action,
                &runtime,
            )
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
