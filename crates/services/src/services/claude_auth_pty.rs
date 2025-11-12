use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing;
use utils::path::get_anyon_temp_dir;
use uuid::Uuid;

use crate::services::secret_store::{SECRET_CLAUDE_ACCESS, SecretStore, SecretStoreError};

const CLAUDE_LOGIN_DEFAULT_CMD: &str = "npx -y @anthropic-ai/claude-code@2.0.31 login";
const MAX_LOG_ENTRIES: usize = 1000;
const MAX_LOG_ENTRY_LENGTH: usize = 2000;

#[derive(Debug, Error)]
pub enum ClaudePtyError {
    #[error("Failed to create PTY: {0}")]
    PtyCreation(String),
    #[error("Failed to spawn process: {0}")]
    Spawn(String),
    #[error("Session not found")]
    SessionNotFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SecretStore(#[from] SecretStoreError),
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaudePtyLogDirection {
    Input,
    Output,
}

#[derive(Clone, Serialize)]
pub struct ClaudePtyLogEntry {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub direction: ClaudePtyLogDirection,
    pub data: String,
}

pub struct ClaudePtySession {
    home_dir: PathBuf,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    logs: Arc<Mutex<Vec<ClaudePtyLogEntry>>>,
}

#[derive(Clone)]
pub struct ClaudePtyManager {
    sessions: Arc<DashMap<Uuid, Arc<ClaudePtySession>>>,
    secret_store: Arc<SecretStore>,
    completed_logs: Arc<DashMap<Uuid, Vec<ClaudePtyLogEntry>>>,
}

impl ClaudePtyManager {
    pub fn new(secret_store: Arc<SecretStore>) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            secret_store,
            completed_logs: Arc::new(DashMap::new()),
        }
    }

    pub async fn start_session(&self, user_id: &str) -> Result<Uuid, ClaudePtyError> {
        let session_id = Uuid::new_v4();

        // Create isolated home directory
        let home_dir = get_anyon_temp_dir()
            .join("claude-auth")
            .join(user_id)
            .join(session_id.to_string());

        std::fs::create_dir_all(&home_dir).map_err(|e| ClaudePtyError::Io(e))?;

        let claude_dir = home_dir.join(".claude");
        std::fs::create_dir_all(&claude_dir).map_err(|e| ClaudePtyError::Io(e))?;

        let xdg_config_home = home_dir.join(".config");
        std::fs::create_dir_all(&xdg_config_home).map_err(|e| ClaudePtyError::Io(e))?;
        let claude_code_xdg_dir = xdg_config_home.join("claude-code");
        std::fs::create_dir_all(&claude_code_xdg_dir).map_err(|e| ClaudePtyError::Io(e))?;

        // Create PTY
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| ClaudePtyError::PtyCreation(e.to_string()))?;

        // Build command
        let parts: Vec<&str> = CLAUDE_LOGIN_DEFAULT_CMD.split_whitespace().collect();
        let mut cmd = CommandBuilder::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }
        cmd.env("HOME", &home_dir);
        cmd.env("TERM", "xterm-256color");
        cmd.env("FORCE_COLOR", "0");
        cmd.env("NPM_CONFIG_COLOR", "false");
        cmd.env("CLAUDE_CONFIG_DIR", &claude_dir);
        cmd.env("XDG_CONFIG_HOME", &xdg_config_home);
        cmd.env("CLAUDE_CODE_CONFIG_DIR", &claude_code_xdg_dir);

        tracing::info!(
            session_id = %session_id,
            home_dir = ?home_dir,
            command = CLAUDE_LOGIN_DEFAULT_CMD,
            "Starting Claude login PTY session"
        );

        // Spawn child process
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| ClaudePtyError::Spawn(e.to_string()))?;

        // Detach child to prevent zombie
        std::mem::drop(child);

        // Get reader and writer
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| ClaudePtyError::PtyCreation(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| ClaudePtyError::PtyCreation(e.to_string()))?;

        let session = Arc::new(ClaudePtySession {
            home_dir: home_dir.clone(),
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
            logs: Arc::new(Mutex::new(Vec::new())),
        });

        self.sessions.insert(session_id, session);

        Ok(session_id)
    }

    pub fn get_session(&self, session_id: &Uuid) -> Result<Arc<ClaudePtySession>, ClaudePtyError> {
        self.sessions
            .get(session_id)
            .map(|s| s.value().clone())
            .ok_or(ClaudePtyError::SessionNotFound)
    }

    pub async fn write_input(&self, session_id: &Uuid, data: &[u8]) -> Result<(), ClaudePtyError> {
        let session = self.get_session(session_id)?;
        let mut writer = session.writer.lock().await;
        writer.write_all(data)?;
        writer.flush()?;
        session.append_log(ClaudePtyLogDirection::Input, data).await;
        Ok(())
    }

    pub async fn read_output(
        &self,
        session_id: &Uuid,
        buf: &mut [u8],
    ) -> Result<usize, ClaudePtyError> {
        let session = self.get_session(session_id)?;
        let mut reader = session.reader.lock().await;
        let n = reader.read(buf)?;
        if n > 0 {
            session
                .append_log(ClaudePtyLogDirection::Output, &buf[..n])
                .await;
        }
        Ok(n)
    }

    pub async fn cancel_session(&self, session_id: &Uuid) -> Result<(), ClaudePtyError> {
        if let Some((_, session)) = self.sessions.remove(session_id) {
            let logs = session.logs.lock().await.clone();
            self.completed_logs.insert(*session_id, logs);
        }
        Ok(())
    }

    pub async fn get_logs(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<ClaudePtyLogEntry>, ClaudePtyError> {
        if let Some(session) = self.sessions.get(session_id) {
            let logs = session.logs.lock().await.clone();
            drop(session);
            return Ok(logs);
        }
        if let Some((_, logs)) = self.completed_logs.remove(session_id) {
            return Ok(logs);
        }
        Ok(Vec::new())
    }

    /// Check if login is successful by looking for Claude credential artifacts
    pub async fn check_login_success(
        &self,
        session_id: &Uuid,
        user_id: &str,
    ) -> Result<bool, ClaudePtyError> {
        let session = self.get_session(session_id)?;

        tracing::debug!(
            session_id = %session_id,
            home_dir = %session.home_dir.display(),
            "🔍 Checking for Claude credential files..."
        );

        let credential_paths = [
            session.home_dir.join(".claude").join(".claude.json"),
            session.home_dir.join(".claude").join("meta.json"),
            session.home_dir.join(".claude").join("config.json"),
            session
                .home_dir
                .join(".config")
                .join("claude-code")
                .join("config.json"),
            session.home_dir.join(".claude.json"),
        ];

        // Log each path check
        for (i, path) in credential_paths.iter().enumerate() {
            let exists = path.exists();
            tracing::debug!(
                session_id = %session_id,
                path_index = i,
                path = %path.display(),
                exists = exists,
                "📂 Checking credential path"
            );
        }

        let credential_path = credential_paths.into_iter().find(|path| path.exists());

        let Some(path) = credential_path else {
            tracing::debug!(
                session_id = %session_id,
                "❌ No credential file found yet"
            );
            return Ok(false);
        };

        tracing::info!(
            session_id = %session_id,
            path = %path.display(),
            "✅ Found credential file!"
        );

        let credential_content = std::fs::read(&path)?;

        tracing::debug!(
            session_id = %session_id,
            content_length = credential_content.len(),
            "📄 Read credential content"
        );

        self.secret_store
            .put_secret(user_id, SECRET_CLAUDE_ACCESS, &credential_content)
            .await?;

        tracing::info!(
            session_id = %session_id,
            user_id = %user_id,
            "💾 Claude login successful, credential stored in secret store"
        );

        Ok(true)
    }
}

impl ClaudePtySession {
    async fn append_log(&self, direction: ClaudePtyLogDirection, data: &[u8]) {
        let text = Self::truncate_log_entry(data);
        let mut logs = self.logs.lock().await;
        logs.push(ClaudePtyLogEntry {
            timestamp: Utc::now(),
            direction,
            data: text,
        });
        if logs.len() > MAX_LOG_ENTRIES {
            let overflow = logs.len() - MAX_LOG_ENTRIES;
            logs.drain(0..overflow);
        }
    }

    fn truncate_log_entry(data: &[u8]) -> String {
        let mut text = String::from_utf8_lossy(data).to_string();
        if text.len() > MAX_LOG_ENTRY_LENGTH {
            text.truncate(MAX_LOG_ENTRY_LENGTH);
            text.push('…');
        }
        text
    }
}
