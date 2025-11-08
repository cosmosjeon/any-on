use std::{env, fs, path::PathBuf};

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub base_dir: PathBuf,
    pub data_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub worktree_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub database_file: PathBuf,
    pub log_file: PathBuf,
    pub docker_user: String,
    pub container_image: String,
}

impl CloudConfig {
    pub fn from_env() -> Self {
        let base_dir = env::var_os("ANYON_CLOUD_BASE_DIR")
            .map(PathBuf::from)
            .or_else(|| env::var_os("ANYON_BASE_DIR").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("/var/opt/anyon"));
        let base_dir = if base_dir.ends_with("anyon") {
            base_dir
        } else {
            base_dir.join("anyon")
        };

        let data_dir = env::var_os("ANYON_ASSET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.join("data"));
        let temp_dir = env::var_os("ANYON_TEMP_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.join("tmp"));
        let worktree_dir = env::var_os("ANYON_WORKTREE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.join("worktrees"));
        let workspace_dir = env::var_os("ANYON_WORKSPACE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.join("workspace"));
        let database_file = env::var_os("ANYON_DATABASE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("anyon.db"));
        let log_file = env::var_os("ANYON_LOG_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| base_dir.join("logs/server.log"));
        let docker_user = env::var("ANYON_DOCKER_USER")
            .or_else(|_| env::var("USER"))
            .unwrap_or_else(|_| "ubuntu".to_string());
        let container_image = env::var("ANYON_CLOUD_CONTAINER_IMAGE")
            .or_else(|_| env::var("ANYON_CONTAINER_IMAGE"))
            .unwrap_or_else(|_| "anyon-claude:latest".to_string());

        Self {
            base_dir,
            data_dir,
            temp_dir,
            worktree_dir,
            workspace_dir,
            database_file,
            log_file,
            docker_user,
            container_image,
        }
    }

    pub fn apply(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("Failed to create data dir: {}", self.data_dir.display()))?;
        fs::create_dir_all(&self.temp_dir)
            .with_context(|| format!("Failed to create temp dir: {}", self.temp_dir.display()))?;
        fs::create_dir_all(&self.worktree_dir).with_context(|| {
            format!(
                "Failed to create worktree dir: {}",
                self.worktree_dir.display()
            )
        })?;
        fs::create_dir_all(&self.workspace_dir).with_context(|| {
            format!(
                "Failed to create workspace dir: {}",
                self.workspace_dir.display()
            )
        })?;
        if let Some(parent) = self.log_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create log dir: {}", parent.display()))?;
        }
        if let Some(parent) = self.database_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database dir: {}", parent.display()))?;
        }

        // std::env::set_var takes OsStr references and may read process-wide state; wrap in unsafe block explicitly.
        unsafe {
            env::set_var("ANYON_ASSET_DIR", &self.data_dir);
            env::set_var("ANYON_TEMP_DIR", &self.temp_dir);
            env::set_var("ANYON_WORKTREE_DIR", &self.worktree_dir);
            env::set_var("ANYON_WORKSPACE_DIR", &self.workspace_dir);
            env::set_var("ANYON_LOG_FILE", &self.log_file);
            env::set_var("ANYON_DOCKER_USER", &self.docker_user);
            env::set_var("ANYON_CLOUD_BASE_DIR", &self.base_dir);
            env::set_var("ANYON_DATABASE_FILE", &self.database_file);
            env::set_var("ANYON_CLOUD_CONTAINER_IMAGE", &self.container_image);
            if env::var_os("DATABASE_URL").is_none() {
                env::set_var(
                    "DATABASE_URL",
                    format!(
                        "sqlite://{}",
                        self.database_file.as_path().to_string_lossy()
                    ),
                );
            }
        }
        Ok(())
    }

    pub fn workspace_dir(&self) -> &PathBuf {
        &self.workspace_dir
    }

    pub fn container_image(&self) -> &str {
        &self.container_image
    }
}
