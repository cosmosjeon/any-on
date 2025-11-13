use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Once},
};

use async_trait::async_trait;
use db::DBService;
use deployment::{Deployment, DeploymentError, WorkspaceDirError};
use executors::profile::ExecutorConfigs;
use services::services::{
    analytics::{AnalyticsConfig, AnalyticsContext, AnalyticsService, generate_user_id},
    approvals::Approvals,
    auth::AuthService,
    claude_auth::ClaudeAuthManager,
    config::{Config, load_config_from_file, save_config_to_file},
    container::ContainerService,
    drafts::DraftsService,
    events::EventService,
    file_search_cache::FileSearchCache,
    filesystem::FilesystemService,
    git::GitService,
    github_user_cache::GitHubUserCache,
    image::ImageService,
    secret_store::{SECRET_GITHUB_OAUTH, SECRET_GITHUB_PAT, SecretStore},
};
use tokio::sync::RwLock;
use utils::{assets::config_path, msg_store::MsgStore};
use uuid::Uuid;

use crate::container::LocalContainerService;
mod command;
pub mod container;

static WORKSPACE_DIR_FALLBACK_LOG: Once = Once::new();

#[derive(Clone)]
pub struct LocalDeployment {
    config: Arc<RwLock<Config>>,
    user_id: String,
    db: DBService,
    analytics: Option<AnalyticsService>,
    msg_stores: Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>>,
    container: LocalContainerService,
    git: GitService,
    auth: AuthService,
    image: ImageService,
    filesystem: FilesystemService,
    events: EventService,
    file_search_cache: Arc<FileSearchCache>,
    approvals: Approvals,
    drafts: DraftsService,
    secret_store: SecretStore,
    claude_auth: ClaudeAuthManager,
    claude_auth_pty: services::services::claude_auth_pty::ClaudePtyManager,
    github_user_cache: GitHubUserCache,
}

#[async_trait]
impl Deployment for LocalDeployment {
    async fn new() -> Result<Self, DeploymentError> {
        let config_path = config_path();
        let mut raw_config = load_config_from_file(&config_path).await;

        let profiles = ExecutorConfigs::get_cached();
        if !raw_config.onboarding_acknowledged
            && let Ok(recommended_executor) = profiles.get_recommended_executor_profile().await
        {
            raw_config.executor_profile = recommended_executor;
        }

        // Check if app version has changed and set release notes flag
        {
            let current_version = utils::version::APP_VERSION;
            let stored_version = raw_config.last_app_version.as_deref();

            if stored_version != Some(current_version) {
                // Show release notes only if this is an upgrade (not first install)
                raw_config.show_release_notes = stored_version.is_some();
                raw_config.last_app_version = Some(current_version.to_string());
            }
        }

        let config = Arc::new(RwLock::new(raw_config));
        let user_id = generate_user_id();
        let analytics = AnalyticsConfig::new().map(AnalyticsService::new);
        let git = GitService::new();
        let msg_stores = Arc::new(RwLock::new(HashMap::new()));
        let auth = AuthService::new();
        let filesystem = FilesystemService::new();

        // Create shared components for EventService
        let events_msg_store = Arc::new(MsgStore::new());
        let events_entry_count = Arc::new(RwLock::new(0));

        // Create DB with event hooks
        let db = {
            let hook = EventService::create_hook(
                events_msg_store.clone(),
                events_entry_count.clone(),
                DBService::new().await?, // Temporary DB service for the hook
            );
            DBService::new_with_after_connect(hook).await?
        };

        let image = ImageService::new(db.clone().pool)?;
        {
            let image_service = image.clone();
            tokio::spawn(async move {
                tracing::info!("Starting orphaned image cleanup...");
                if let Err(e) = image_service.delete_orphaned_images().await {
                    tracing::error!("Failed to clean up orphaned images: {}", e);
                }
            });
        }

        let secret_store = SecretStore::new(db.clone())?;

        {
            let mut config_guard = config.write().await;
            Self::migrate_github_secrets(&secret_store, &user_id, &mut config_guard.github).await?;
            save_config_to_file(&config_guard, &config_path).await?;
        }

        let claude_auth = ClaudeAuthManager::new(secret_store.clone(), user_id.clone());
        let claude_auth_pty = services::services::claude_auth_pty::ClaudePtyManager::new(Arc::new(
            secret_store.clone(),
        ));

        let approvals = Approvals::new(msg_stores.clone());

        // We need to make analytics accessible to the ContainerService
        // TODO: Handle this more gracefully
        let analytics_ctx = analytics.as_ref().map(|s| AnalyticsContext {
            user_id: user_id.clone(),
            analytics_service: s.clone(),
        });
        let container = LocalContainerService::new(
            db.clone(),
            msg_stores.clone(),
            config.clone(),
            git.clone(),
            image.clone(),
            analytics_ctx,
            approvals.clone(),
        );
        container.spawn_worktree_cleanup().await;

        let events = EventService::new(db.clone(), events_msg_store, events_entry_count);
        let drafts = DraftsService::new(db.clone(), image.clone());
        let file_search_cache = Arc::new(FileSearchCache::new());
        let github_user_cache = GitHubUserCache::new();

        Ok(Self {
            config,
            user_id,
            db,
            analytics,
            msg_stores,
            container,
            git,
            auth,
            image,
            filesystem,
            events,
            file_search_cache,
            approvals,
            drafts,
            secret_store,
            claude_auth,
            claude_auth_pty,
            github_user_cache,
        })
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn shared_types() -> Vec<String> {
        vec![]
    }

    fn config(&self) -> &Arc<RwLock<Config>> {
        &self.config
    }

    fn db(&self) -> &DBService {
        &self.db
    }

    fn analytics(&self) -> &Option<AnalyticsService> {
        &self.analytics
    }

    fn container(&self) -> &impl ContainerService {
        &self.container
    }
    fn auth(&self) -> &AuthService {
        &self.auth
    }

    fn git(&self) -> &GitService {
        &self.git
    }

    fn image(&self) -> &ImageService {
        &self.image
    }

    fn filesystem(&self) -> &FilesystemService {
        &self.filesystem
    }

    fn msg_stores(&self) -> &Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>> {
        &self.msg_stores
    }

    fn events(&self) -> &EventService {
        &self.events
    }

    fn file_search_cache(&self) -> &Arc<FileSearchCache> {
        &self.file_search_cache
    }

    fn approvals(&self) -> &Approvals {
        &self.approvals
    }

    fn drafts(&self) -> &DraftsService {
        &self.drafts
    }

    fn secret_store(&self) -> &SecretStore {
        &self.secret_store
    }

    fn claude_auth(&self) -> &ClaudeAuthManager {
        &self.claude_auth
    }

    fn claude_auth_pty(&self) -> &services::services::claude_auth_pty::ClaudePtyManager {
        &self.claude_auth_pty
    }

    fn github_user_cache(&self) -> &GitHubUserCache {
        &self.github_user_cache
    }
}

impl LocalDeployment {
    /// Get workspace directory path
    /// Uses config.workspace_dir if set, otherwise defaults to ~/workspace (or equivalent)
    pub async fn workspace_dir(&self) -> Result<PathBuf, WorkspaceDirError> {
        if let Some(explicit_dir) = {
            let config = self.config.read().await;
            config.workspace_dir.clone()
        } {
            return Ok(PathBuf::from(explicit_dir));
        }

        let default_dir = Self::default_workspace_dir()?;

        {
            let mut config = self.config.write().await;
            if let Some(existing) = config.workspace_dir.clone() {
                return Ok(PathBuf::from(existing));
            }
            config.workspace_dir = Some(default_dir.to_string_lossy().into_owned());
        }

        let log_path = format!("{}", default_dir.display());
        WORKSPACE_DIR_FALLBACK_LOG.call_once(move || {
            tracing::info!(
                workspace_dir = %log_path,
                "Workspace directory not configured; using default path."
            );
            tracing::info!(
                "Set `workspace_dir` in dev_assets/config.json to choose a custom workspace location."
            );
        });

        Ok(default_dir)
    }

    fn default_workspace_dir() -> Result<PathBuf, WorkspaceDirError> {
        if let Some(home) = std::env::var_os("HOME").filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(home).join("workspace"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE").filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(profile).join("workspace"));
        }

        Err(WorkspaceDirError::MissingHomeEnvironment)
    }

    /// Expose the underlying local container service so other deployments can
    /// compose additional behavior (e.g., cloud wrappers) without re-building
    /// the entire stack.
    pub fn local_container_service(&self) -> &LocalContainerService {
        &self.container
    }

    async fn migrate_github_secrets(
        secret_store: &SecretStore,
        user_id: &str,
        github_config: &mut services::services::config::GitHubConfig,
    ) -> Result<(), DeploymentError> {
        if let Some(token) = github_config.oauth_token.take() {
            if token.is_empty() {
                secret_store
                    .delete_secret(user_id, SECRET_GITHUB_OAUTH)
                    .await?;
            } else {
                secret_store
                    .put_secret(user_id, SECRET_GITHUB_OAUTH, token.as_bytes())
                    .await?;
            }
        }
        if let Some(pat) = github_config.pat.take() {
            if pat.is_empty() {
                secret_store
                    .delete_secret(user_id, SECRET_GITHUB_PAT)
                    .await?;
            } else {
                secret_store
                    .put_secret(user_id, SECRET_GITHUB_PAT, pat.as_bytes())
                    .await?;
            }
        }
        Ok(())
    }
}
