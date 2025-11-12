use std::{fmt, time::Duration};

use anyhow::{Context, Result, anyhow};
use bollard::{
    Docker,
    container::{
        Config as ContainerConfig, CreateContainerOptions, InspectContainerOptions, LogOutput,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions, WaitContainerOptions,
    },
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    models::{ContainerInspectResponse, ContainerState, HostConfig},
};
use futures::{Stream, StreamExt, TryStreamExt};
use tokio::time::sleep;

const DEFAULT_PULL_RETRY: usize = 3;

/// Thin wrapper that centralizes helper utilities shared across the Docker POC tests.
pub struct DockerHarness {
    docker: Docker,
}

impl DockerHarness {
    pub async fn connect() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to local Docker daemon")?;
        // Probe the daemon to ensure the connection is healthy before running expensive tests.
        docker
            .version()
            .await
            .context("Failed to fetch Docker version")?;
        Ok(Self { docker })
    }

    pub fn client(&self) -> &Docker {
        &self.docker
    }

    /// Ensure an image exists locally, pulling it if missing.
    pub async fn ensure_image(&self, image: &str) -> Result<()> {
        // First check if image exists locally
        if self.docker.inspect_image(image).await.is_ok() {
            tracing::debug!("Image {image} already exists locally, skipping pull");
            return Ok(());
        }

        // Image not found locally, try to pull it
        tracing::info!("Image {image} not found locally, attempting to pull...");
        let mut retries = DEFAULT_PULL_RETRY;
        loop {
            match self
                .docker
                .create_image(
                    Some(CreateImageOptions {
                        from_image: image,
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .try_collect::<Vec<_>>()
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) if retries > 0 => {
                    retries -= 1;
                    tracing::warn!(
                        "Failed to pull image {image} ({e:?}), retrying {} more times",
                        retries
                    );
                    sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to pull image {image} after retries: {e}"
                    ));
                }
            }
        }
    }

    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
        cmd: Option<Vec<String>>,
        host_config: Option<HostConfig>,
        env: Option<Vec<String>>,
        attach: bool,
    ) -> Result<String> {
        let mut config = ContainerConfig::<String> {
            image: Some(image.to_string()),
            cmd,
            env,
            host_config,
            ..Default::default()
        };

        if attach {
            config.attach_stdout = Some(true);
            config.attach_stderr = Some(true);
            config.tty = Some(false);
        }

        let options = Some(CreateContainerOptions {
            name,
            platform: None,
        });

        let response = self
            .docker
            .create_container(options, config)
            .await
            .with_context(|| format!("Failed to create container {name}"))?;

        Ok(response.id)
    }

    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .with_context(|| format!("Failed to start container {id}"))?;
        Ok(())
    }

    pub async fn stop_container(&self, id: &str) -> Result<()> {
        self.stop_container_with_timeout(id, Duration::from_secs(1))
            .await
    }

    pub async fn stop_container_with_timeout(&self, id: &str, timeout: Duration) -> Result<()> {
        let seconds: i64 = timeout.as_secs().try_into().unwrap_or(i64::MAX);
        self.docker
            .stop_container(
                id,
                Some(StopContainerOptions {
                    t: seconds,
                    ..Default::default()
                }),
            )
            .await
            .with_context(|| format!("Failed to stop container {id}"))
    }

    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        self.docker
            .remove_container(
                id,
                Some(RemoveContainerOptions {
                    force,
                    v: true,
                    ..Default::default()
                }),
            )
            .await
            .with_context(|| format!("Failed to remove container {id}"))
    }

    pub async fn wait_container(&self, id: &str) -> Result<ContainerExit> {
        let mut wait_stream = self
            .docker
            .wait_container(id, None::<WaitContainerOptions<String>>);
        while let Some(status) = wait_stream.next().await {
            match status {
                Ok(status) => {
                    return self
                        .resolve_exit_state(
                            id,
                            Some(status.status_code),
                            status.error.and_then(|err| err.message),
                        )
                        .await;
                }
                Err(err) => {
                    return self
                        .resolve_exit_state(id, None, Some(err.to_string()))
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to wait for container {id} completion (wait error: {err})"
                            )
                        });
                }
            }
        }
        self.resolve_exit_state(id, None, None)
            .await
            .with_context(|| {
                format!("Docker wait stream ended before reporting an exit code for container {id}")
            })
    }

    async fn resolve_exit_state(
        &self,
        id: &str,
        wait_code: Option<i64>,
        wait_error: Option<String>,
    ) -> Result<ContainerExit> {
        let inspect = self.inspect_container(id).await?;
        let state = inspect
            .state
            .ok_or_else(|| anyhow!("Container {id} missing state information from inspect"))?;

        let ContainerExitState {
            exit_code,
            error,
            finished_at,
            oom_killed,
        } = ContainerExitState::from_state(state);

        let status_code = wait_code
            .or(exit_code)
            .ok_or_else(|| anyhow!("Container {id} did not report an exit code"))?;

        Ok(ContainerExit {
            status_code,
            error: wait_error.or(error),
            finished_at,
            oom_killed,
        })
    }

    pub async fn inspect_container(&self, id: &str) -> Result<ContainerInspectResponse> {
        self.docker
            .inspect_container(id, None::<InspectContainerOptions>)
            .await
            .with_context(|| format!("Failed to inspect container {id}"))
    }

    pub async fn assert_local_image(&self, image: &str) -> Result<()> {
        self.docker
            .inspect_image(image)
            .await
            .with_context(|| format!("Docker image {image} not found locally"))
            .map(|_| ())
    }

    pub async fn stream_logs(
        &self,
        id: &str,
        follow: bool,
    ) -> Result<impl Stream<Item = Result<LogChunk>>> {
        let stream = self
            .docker
            .logs(
                id,
                Some(bollard::container::LogsOptions::<String> {
                    follow,
                    stdout: true,
                    stderr: true,
                    tail: "all".into(),
                    ..Default::default()
                }),
            )
            .map_ok(|entry| match entry {
                LogOutput::StdOut { message } => {
                    LogChunk::Stdout(String::from_utf8_lossy(&message).to_string())
                }
                LogOutput::StdErr { message } => {
                    LogChunk::Stderr(String::from_utf8_lossy(&message).to_string())
                }
                LogOutput::Console { message } => {
                    LogChunk::Console(String::from_utf8_lossy(&message).to_string())
                }
                LogOutput::StdIn { .. } => LogChunk::Console(String::new()),
            })
            .map_err(anyhow::Error::from);

        Ok(stream)
    }

    pub async fn exec(
        &self,
        container_id: &str,
        cmd: Vec<&str>,
        attach_stdout: bool,
        attach_stderr: bool,
    ) -> Result<StartExecResults> {
        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdout: Some(attach_stdout),
                    attach_stderr: Some(attach_stderr),
                    cmd: Some(cmd.into_iter().map(String::from).collect()),
                    ..Default::default()
                },
            )
            .await?;

        self.docker
            .start_exec(&exec.id, None::<bollard::exec::StartExecOptions>)
            .await
            .context("Failed to start exec instance")
    }
}

#[derive(Debug, Clone)]
pub struct ContainerExit {
    pub status_code: i64,
    pub error: Option<String>,
    pub finished_at: Option<String>,
    pub oom_killed: bool,
}

impl ContainerExit {
    pub fn succeeded(&self) -> bool {
        self.status_code == 0
    }

    pub fn was_signaled(&self) -> bool {
        (128..=255).contains(&self.status_code)
    }
}

struct ContainerExitState {
    exit_code: Option<i64>,
    error: Option<String>,
    finished_at: Option<String>,
    oom_killed: bool,
}

impl ContainerExitState {
    fn from_state(state: ContainerState) -> Self {
        Self {
            exit_code: state.exit_code,
            error: state.error,
            finished_at: state.finished_at,
            oom_killed: state.oom_killed.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogChunk {
    Stdout(String),
    Stderr(String),
    Console(String),
}

impl fmt::Display for LogChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogChunk::Stdout(msg) => write!(f, "STDOUT: {msg}"),
            LogChunk::Stderr(msg) => write!(f, "STDERR: {msg}"),
            LogChunk::Console(msg) => write!(f, "CONSOLE: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use bollard::models::HostConfig;
    use futures::StreamExt;
    use tokio::time::{Duration, Instant};
    use uuid::Uuid;

    use super::*;

    const BASE_IMAGE: &str = "ubuntu:22.04";
    const CLAUDE_IMAGE: &str = "anyon-claude:latest";

    struct ContainerGuard {
        docker: Docker,
        id: Option<String>,
    }

    impl ContainerGuard {
        fn new(harness: &DockerHarness, id: String) -> Self {
            Self {
                docker: harness.client().clone(),
                id: Some(id),
            }
        }

        async fn cleanup(mut self) -> Result<()> {
            if let Some(id) = self.id.take() {
                self.docker
                    .remove_container(
                        &id,
                        Some(RemoveContainerOptions {
                            force: true,
                            v: true,
                            ..Default::default()
                        }),
                    )
                    .await
                    .context("Failed to remove container during cleanup")?;
            }
            Ok(())
        }
    }

    impl Drop for ContainerGuard {
        fn drop(&mut self) {
            if let Some(id) = self.id.take() {
                let docker = self.docker.clone();
                tokio::spawn(async move {
                    let _ = docker
                        .remove_container(
                            &id,
                            Some(RemoveContainerOptions {
                                force: true,
                                v: true,
                                ..Default::default()
                            }),
                        )
                        .await;
                });
            }
        }
    }

    fn unique_name(label: &str) -> String {
        format!("{label}-{}", Uuid::new_v4())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_docker_basic() -> Result<()> {
        let harness = DockerHarness::connect().await?;
        let version = harness.client().version().await?;
        assert!(
            version
                .version
                .as_deref()
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            "Docker daemon did not return a semantic version"
        );
        assert!(
            version
                .api_version
                .as_deref()
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            "Docker daemon did not report API version"
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_container_echo_logs() -> Result<()> {
        let harness = DockerHarness::connect().await?;
        harness.ensure_image(BASE_IMAGE).await?;

        let cmd = vec![
            "/bin/bash".into(),
            "-c".into(),
            "echo hello-from-docker".into(),
        ];

        let container_id = harness
            .create_container(
                &unique_name("docker-poc-echo"),
                BASE_IMAGE,
                Some(cmd),
                None,
                None,
                true,
            )
            .await?;
        let guard = ContainerGuard::new(&harness, container_id.clone());

        harness.start_container(&container_id).await?;
        let mut stream = harness.stream_logs(&container_id, true).await?;
        let mut stdout_buf = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk? {
                LogChunk::Stdout(line) => stdout_buf.push_str(&line),
                LogChunk::Stderr(line) => panic!("stderr emitted during echo test: {line}"),
                LogChunk::Console(_) => {}
            }
        }
        let exit = harness.wait_container(&container_id).await?;
        guard.cleanup().await?;
        assert!(
            stdout_buf.contains("hello-from-docker"),
            "stdout missing expected text: {stdout_buf}"
        );
        assert_eq!(
            exit.status_code, 0,
            "Echo container exited with non-zero status: {}",
            exit.status_code
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_claude_image_reports_version() -> Result<()> {
        let harness = DockerHarness::connect().await?;
        harness.assert_local_image(CLAUDE_IMAGE).await?;

        let cmd = vec!["claude".into(), "--version".into()];
        let container_id = harness
            .create_container(
                &unique_name("docker-poc-claude-version"),
                CLAUDE_IMAGE,
                Some(cmd),
                None,
                None,
                true,
            )
            .await?;
        let guard = ContainerGuard::new(&harness, container_id.clone());

        harness.start_container(&container_id).await?;
        let mut stream = harness.stream_logs(&container_id, true).await?;
        let mut captured = String::new();
        while let Some(chunk) = stream.next().await {
            if let LogChunk::Stdout(line) = chunk? {
                captured.push_str(&line);
            }
        }
        let exit = harness.wait_container(&container_id).await?;
        guard.cleanup().await?;
        assert!(
            captured.contains("Claude Code"),
            "claude --version output unexpected: {captured}"
        );
        assert_eq!(
            exit.status_code, 0,
            "Claude version container exited with non-zero status: {}",
            exit.status_code
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_streaming_and_forced_shutdown() -> Result<()> {
        let harness = DockerHarness::connect().await?;
        harness.ensure_image(BASE_IMAGE).await?;

        // Countdown stream should emit roughly once per second.
        let countdown_cmd = vec![
            "bash".into(),
            "-c".into(),
            "for i in {5..1}; do echo $i; sleep 1; done".into(),
        ];
        let countdown_id = harness
            .create_container(
                &unique_name("docker-poc-countdown"),
                BASE_IMAGE,
                Some(countdown_cmd),
                None,
                None,
                true,
            )
            .await?;
        let countdown_guard = ContainerGuard::new(&harness, countdown_id.clone());
        harness.start_container(&countdown_id).await?;
        let mut stream = harness.stream_logs(&countdown_id, true).await?;
        let mut entries: Vec<(Instant, String)> = Vec::new();
        while let Some(chunk) = stream.next().await {
            if let LogChunk::Stdout(line) = chunk? {
                for segment in line.lines() {
                    let trimmed = segment.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    entries.push((Instant::now(), trimmed.to_string()));
                }
            }
        }
        let countdown_exit = harness.wait_container(&countdown_id).await?;
        countdown_guard.cleanup().await?;
        assert_eq!(
            countdown_exit.status_code, 0,
            "Countdown container exited with non-zero status: {}",
            countdown_exit.status_code
        );

        assert!(
            entries.len() >= 5,
            "Expected at least 5 countdown entries, got {}",
            entries.len()
        );
        for pair in entries.windows(2) {
            let delta = pair[1].0 - pair[0].0;
            assert!(
                delta >= Duration::from_millis(600) && delta <= Duration::from_millis(1400),
                "Countdown stream not near real-time cadence: {:?}",
                delta
            );
        }

        // Second container: verify we can stop a long-running job mid-stream.
        let long_cmd = vec![
            "bash".into(),
            "-c".into(),
            "for i in {1..10}; do echo tick; sleep 1; done".into(),
        ];
        let long_id = harness
            .create_container(
                &unique_name("docker-poc-cancel"),
                BASE_IMAGE,
                Some(long_cmd),
                None,
                None,
                true,
            )
            .await?;
        let long_guard = ContainerGuard::new(&harness, long_id.clone());
        harness.start_container(&long_id).await?;
        let mut stream = harness.stream_logs(&long_id, true).await?;
        let mut ticks = 0usize;
        let mut should_break = false;
        while let Some(chunk) = stream.next().await {
            if let LogChunk::Stdout(line) = chunk? {
                for segment in line.lines() {
                    if segment.contains("tick") {
                        ticks += 1;
                        if ticks == 3 {
                            harness.stop_container(&long_id).await?;
                            should_break = true;
                            break;
                        }
                    }
                }
                if should_break {
                    break;
                }
            }
        }
        let cancel_exit = harness.wait_container(&long_id).await?;
        long_guard.cleanup().await?;
        assert!(
            ticks == 3,
            "Expected to capture exactly 3 ticks before cancellation"
        );
        assert!(
            cancel_exit.was_signaled(),
            "Expected forced shutdown to report a signal exit, got code {}",
            cancel_exit.status_code
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_resource_limits_applied() -> Result<()> {
        let harness = DockerHarness::connect().await?;
        harness.ensure_image(BASE_IMAGE).await?;

        let host_config = HostConfig {
            memory: Some(2 * 1024 * 1024 * 1024),
            nano_cpus: Some(1_000_000_000),
            ..Default::default()
        };
        let cmd = vec!["/bin/bash".into(), "-c".into(), "sleep 1".into()];
        let container_id = harness
            .create_container(
                &unique_name("docker-poc-limits"),
                BASE_IMAGE,
                Some(cmd),
                Some(host_config),
                None,
                true,
            )
            .await?;
        let guard = ContainerGuard::new(&harness, container_id.clone());
        harness.start_container(&container_id).await?;
        let exit = harness.wait_container(&container_id).await?;
        let inspect = harness.inspect_container(&container_id).await?;
        guard.cleanup().await?;
        assert_eq!(
            exit.status_code, 0,
            "Resource limit container exited with non-zero status: {}",
            exit.status_code
        );

        let cfg = inspect
            .host_config
            .expect("host config missing from inspect output");
        assert_eq!(
            cfg.memory,
            Some(2 * 1024 * 1024 * 1024),
            "Memory limit not applied"
        );
        assert_eq!(cfg.nano_cpus, Some(1_000_000_000), "CPU limit not applied");
        Ok(())
    }
}
