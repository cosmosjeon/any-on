use dashmap::DashMap;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::{Mutex, mpsc};
use tracing;
use utils::path::get_anyon_temp_dir;
use uuid::Uuid;

use crate::services::secret_store::{SECRET_CLAUDE_ACCESS, SecretStore, SecretStoreError};

const CLAUDE_LOGIN_DEFAULT_CMD: &str = "npx -y @anthropic-ai/claude-code@2.0.31 login";

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

pub struct ClaudePtySession {
    session_id: Uuid,
    home_dir: PathBuf,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
}

#[derive(Clone)]
pub struct ClaudePtyManager {
    sessions: Arc<DashMap<Uuid, Arc<ClaudePtySession>>>,
    secret_store: Arc<SecretStore>,
}

impl ClaudePtyManager {
    pub fn new(secret_store: Arc<SecretStore>) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            secret_store,
        }
    }

    pub async fn start_session(&self, user_id: &str) -> Result<Uuid, ClaudePtyError> {
        let session_id = Uuid::new_v4();

        // Create isolated home directory
        let home_dir = get_anyon_temp_dir()
            .join("claude-auth")
            .join(session_id.to_string());

        std::fs::create_dir_all(&home_dir)
            .map_err(|e| ClaudePtyError::Io(e))?;

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

        tracing::info!(
            session_id = %session_id,
            home_dir = ?home_dir,
            command = CLAUDE_LOGIN_DEFAULT_CMD,
            "Starting Claude login PTY session"
        );

        // Spawn child process
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| ClaudePtyError::Spawn(e.to_string()))?;

        // Detach child to prevent zombie
        std::mem::drop(child);

        // Get reader and writer
        let reader = pair.master.try_clone_reader()
            .map_err(|e| ClaudePtyError::PtyCreation(e.to_string()))?;
        let writer = pair.master.take_writer()
            .map_err(|e| ClaudePtyError::PtyCreation(e.to_string()))?;

        let session = Arc::new(ClaudePtySession {
            session_id,
            home_dir: home_dir.clone(),
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
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

    pub async fn write_input(
        &self,
        session_id: &Uuid,
        data: &[u8],
    ) -> Result<(), ClaudePtyError> {
        let session = self.get_session(session_id)?;
        let mut writer = session.writer.lock().await;
        writer.write_all(data)?;
        writer.flush()?;
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
        Ok(n)
    }

    pub async fn cancel_session(&self, session_id: &Uuid) -> Result<(), ClaudePtyError> {
        self.sessions.remove(session_id);
        Ok(())
    }

    /// Check if login is successful by looking for .claude/config.json
    pub async fn check_login_success(
        &self,
        session_id: &Uuid,
        user_id: &str,
    ) -> Result<bool, ClaudePtyError> {
        let session = self.get_session(session_id)?;
        let config_path = session.home_dir.join(".claude").join("config.json");

        if !config_path.exists() {
            return Ok(false);
        }

        // Read the config file and store in SecretStore
        let config_content = std::fs::read(&config_path)?;

        self.secret_store
            .put_secret(
                user_id,
                SECRET_CLAUDE_ACCESS,
                &config_content,
            )
            .await?;

        tracing::info!(
            session_id = %session_id,
            user_id = %user_id,
            "Claude login successful, credential stored"
        );

        Ok(true)
    }
}
