use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use serde::Serialize;
use shlex::Shlex;
use thiserror::Error;
use tokio::{
    fs as async_fs,
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{Mutex, broadcast, mpsc},
    task::JoinHandle,
};
use tokio_stream::wrappers::BroadcastStream;
use utils::path::get_anyon_temp_dir;
use uuid::Uuid;

use crate::services::secret_store::{SECRET_CLAUDE_ACCESS, SecretStore, SecretStoreError};

const CLAUDE_LOGIN_DEFAULT_CMD: &str = "unbuffer npx -y @anthropic-ai/claude-code@2.0.31 login";

#[derive(Debug, Error)]
pub enum ClaudeAuthError {
    #[error("Claude login command not configured")]
    CommandNotConfigured,
    #[error("Failed to start Claude CLI: {0}")]
    Spawn(String),
    #[error("Session not found")]
    SessionNotFound,
    #[error("CLI input channel closed")]
    InputClosed,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SecretStore(#[from] SecretStoreError),
    #[error("Claude CLI did not emit credentials")]
    MissingCredentials,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaudeAuthEvent {
    Output { line: String },
    Completed { success: bool },
    Error { message: String },
}

struct ClaudeSession {
    stdin_tx: mpsc::Sender<String>,
    events_tx: broadcast::Sender<ClaudeAuthEvent>,
    child: Arc<Mutex<Option<Child>>>,
    stdin_handle: Arc<Mutex<Option<ChildStdin>>>,
    home_dir: PathBuf,
    _stdout_task: JoinHandle<()>,
    _stderr_task: JoinHandle<()>,
    _stdin_task: JoinHandle<()>,
    _monitor_task: JoinHandle<()>,
}

#[derive(Clone)]
pub struct ClaudeAuthManager {
    sessions: Arc<DashMap<Uuid, ClaudeSession>>,
    secret_store: SecretStore,
    user_id: String,
    command: Vec<String>,
}

impl ClaudeAuthManager {
    pub fn new(secret_store: SecretStore, user_id: String) -> Self {
        let command = std::env::var("CLAUDE_LOGIN_COMMAND")
            .ok()
            .and_then(|raw| Self::parse_command(&raw).ok())
            .unwrap_or_else(|| Self::parse_command(CLAUDE_LOGIN_DEFAULT_CMD).unwrap());

        Self {
            sessions: Arc::new(DashMap::new()),
            secret_store,
            user_id,
            command,
        }
    }

    fn parse_command(raw: &str) -> Result<Vec<String>, ClaudeAuthError> {
        let mut parser = Shlex::new(raw);
        let mut parts = Vec::new();
        while let Some(token) = parser.next() {
            parts.push(token);
        }
        if parts.is_empty() {
            return Err(ClaudeAuthError::CommandNotConfigured);
        }
        Ok(parts)
    }

    pub async fn start_session(&self) -> Result<Uuid, ClaudeAuthError> {
        let session_id = Uuid::new_v4();
        let home_dir = self.prepare_home_dir(&session_id)?;
        let mut command = self.build_command(&home_dir)?;

        tracing::info!(
            session_id = %session_id,
            home_dir = %home_dir.display(),
            command = ?self.command,
            "Starting Claude login session"
        );

        command.kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|err| {
                tracing::error!(
                    session_id = %session_id,
                    error = %err,
                    "Failed to spawn Claude CLI process"
                );
                ClaudeAuthError::Spawn(err.to_string())
            })?;

        tracing::debug!(
            session_id = %session_id,
            "Claude CLI process spawned successfully"
        );

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ClaudeAuthError::Other("Failed to capture stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ClaudeAuthError::Other("Failed to capture stderr".into()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ClaudeAuthError::Other("Failed to capture stdin".into()))?;

        let (stdin_tx, stdin_rx) = mpsc::channel::<String>(32);
        let (events_tx, _) = broadcast::channel::<ClaudeAuthEvent>(256);
        let child = Arc::new(Mutex::new(Some(child)));
        let stdin_handle = Arc::new(Mutex::new(Some(stdin)));

        let stdout_task = tokio::spawn(Self::stream_pipe(stdout, events_tx.clone()));
        let stderr_task = tokio::spawn(Self::stream_pipe(stderr, events_tx.clone()));
        let stdin_task = tokio::spawn(Self::pump_stdin(stdin_handle.clone(), stdin_rx));
        let monitor_task = self.spawn_monitor(
            session_id,
            child.clone(),
            stdin_handle.clone(),
            events_tx.clone(),
            home_dir.clone(),
        );

        self.sessions.insert(
            session_id,
            ClaudeSession {
                stdin_tx,
                events_tx,
                child,
                stdin_handle,
                home_dir,
                _stdout_task: stdout_task,
                _stderr_task: stderr_task,
                _stdin_task: stdin_task,
                _monitor_task: monitor_task,
            },
        );

        Ok(session_id)
    }

    pub fn subscribe(
        &self,
        session_id: &Uuid,
    ) -> Result<BroadcastStream<ClaudeAuthEvent>, ClaudeAuthError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(ClaudeAuthError::SessionNotFound)?;
        Ok(BroadcastStream::new(session.events_tx.subscribe()))
    }

    pub async fn send_input(
        &self,
        session_id: &Uuid,
        input: String,
    ) -> Result<(), ClaudeAuthError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(ClaudeAuthError::SessionNotFound)?;
        session
            .stdin_tx
            .send(input)
            .await
            .map_err(|_| ClaudeAuthError::InputClosed)
    }

    pub async fn cancel_session(&self, session_id: &Uuid) -> Result<(), ClaudeAuthError> {
        let session = self
            .sessions
            .remove(session_id)
            .map(|(_, session)| session)
            .ok_or(ClaudeAuthError::SessionNotFound)?;

        {
            let mut guard = session.child.lock().await;
            if let Some(mut child) = guard.take() {
                let _ = child.kill().await;
            }
        }

        let _ = session.events_tx.send(ClaudeAuthEvent::Error {
            message: "Session cancelled".into(),
        });
        let _ = async_fs::remove_dir_all(&session.home_dir).await;
        Ok(())
    }

    fn prepare_home_dir(&self, session_id: &Uuid) -> Result<PathBuf, ClaudeAuthError> {
        let base = get_anyon_temp_dir().join("claude-auth");
        fs::create_dir_all(&base)?;
        let session_dir = base.join(session_id.to_string());
        fs::create_dir_all(&session_dir)?;
        Ok(session_dir)
    }

    fn build_command(&self, home_dir: &Path) -> Result<Command, ClaudeAuthError> {
        let mut parts = self.command.clone();
        if parts.is_empty() {
            return Err(ClaudeAuthError::CommandNotConfigured);
        }
        let program = parts.remove(0);
        let mut command = Command::new(program);
        command.args(parts);
        command.env("HOME", home_dir);
        // Force unbuffered output for npx and Node.js processes
        command.env("NODE_NO_WARNINGS", "1");
        command.env("FORCE_COLOR", "0");
        command.env("NPM_CONFIG_COLOR", "false");
        // Disable Node.js output buffering
        command.env("NODE_OPTIONS", "--no-warnings");
        command.env("UV_NO_WARNINGS", "1");
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        Ok(command)
    }

    fn spawn_monitor(
        &self,
        session_id: Uuid,
        child: Arc<Mutex<Option<Child>>>,
        stdin_handle: Arc<Mutex<Option<ChildStdin>>>,
        events_tx: broadcast::Sender<ClaudeAuthEvent>,
        home_dir: PathBuf,
    ) -> JoinHandle<()> {
        let secret_store = self.secret_store.clone();
        let user_id = self.user_id.clone();
        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let status = {
                let mut guard = child.lock().await;
                if let Some(mut child) = guard.take() {
                    tracing::debug!(session_id = %session_id, "Waiting for Claude CLI process to exit");
                    match child.wait().await {
                        Ok(status) => {
                            tracing::info!(
                                session_id = %session_id,
                                exit_code = ?status.code(),
                                "Claude CLI process exited"
                            );
                            status
                        }
                        Err(err) => {
                            tracing::error!(
                                session_id = %session_id,
                                error = %err,
                                "Claude CLI process wait failed"
                            );
                            let _ = events_tx.send(ClaudeAuthEvent::Error {
                                message: format!("Claude CLI failed: {err}"),
                            });
                            let _ = async_fs::remove_dir_all(&home_dir).await;
                            return;
                        }
                    }
                } else {
                    tracing::warn!(session_id = %session_id, "Child process already taken");
                    return;
                }
            };

            let _ = stdin_handle.lock().await.take();

            if status.success() {
                match Self::persist_credentials(&secret_store, &user_id, &home_dir).await {
                    Ok(()) => {
                        let _ = events_tx.send(ClaudeAuthEvent::Completed { success: true });
                    }
                    Err(err) => {
                        let _ = events_tx.send(ClaudeAuthEvent::Error {
                            message: format!("Failed to store Claude credentials: {err}"),
                        });
                    }
                }
            } else {
                let _ = events_tx.send(ClaudeAuthEvent::Error {
                    message: format!("Claude CLI exited with status {status}"),
                });
            }

            sessions.remove(&session_id);
            let _ = async_fs::remove_dir_all(&home_dir).await;
        })
    }

    async fn persist_credentials(
        secret_store: &SecretStore,
        user_id: &str,
        home_dir: &Path,
    ) -> Result<(), ClaudeAuthError> {
        let meta_path = home_dir.join(".claude/meta.json");
        let data = async_fs::read(&meta_path)
            .await
            .map_err(|_| ClaudeAuthError::MissingCredentials)?;
        secret_store
            .put_secret(user_id, SECRET_CLAUDE_ACCESS, &data)
            .await?;
        Ok(())
    }

    async fn stream_pipe<R>(pipe: R, events_tx: broadcast::Sender<ClaudeAuthEvent>)
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        let reader = BufReader::new(pipe);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            tracing::debug!("CLI output line: {}", line);
            let _ = events_tx.send(ClaudeAuthEvent::Output { line });
        }
        tracing::debug!("CLI pipe closed");
    }

    async fn pump_stdin(handle: Arc<Mutex<Option<ChildStdin>>>, mut rx: mpsc::Receiver<String>) {
        while let Some(input) = rx.recv().await {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }
            tracing::info!(input = %trimmed, "Sending input to Claude CLI");
            let mut guard = handle.lock().await;
            if let Some(stdin) = guard.as_mut() {
                if stdin
                    .write_all(format!("{trimmed}\n").as_bytes())
                    .await
                    .is_err()
                {
                    tracing::error!("Failed to write to Claude CLI stdin");
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    tracing::error!(error = %e, "Failed to flush Claude CLI stdin");
                    break;
                }
                tracing::info!("Successfully sent input to Claude CLI");
            } else {
                tracing::warn!("Claude CLI stdin handle is None");
                break;
            }
        }
    }
}
