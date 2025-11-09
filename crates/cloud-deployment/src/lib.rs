use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use db::DBService;
use deployment::{Deployment, DeploymentError};
use local_deployment::{LocalDeployment, container::LocalContainerService};
use services::services::{
    analytics::AnalyticsService,
    approvals::Approvals,
    auth::AuthService,
    cloud_container::{CloudContainerService, CloudContainerSettings},
    config::Config,
    container::ContainerService,
    drafts::DraftsService,
    events::EventService,
    file_search_cache::FileSearchCache,
    filesystem::FilesystemService,
    git::GitService,
    image::ImageService,
    claude_auth::ClaudeAuthManager,
    secret_store::SecretStore,
};
use tokio::sync::RwLock;
use utils::msg_store::MsgStore;
use uuid::Uuid;

mod config;

use config::CloudConfig;

#[derive(Clone)]
pub struct CloudDeployment {
    inner: LocalDeployment,
    cloud_config: CloudConfig,
    container: CloudContainerService<LocalContainerService>,
}

impl CloudDeployment {
    pub fn inner(&self) -> &LocalDeployment {
        &self.inner
    }

    pub fn cloud_config(&self) -> &CloudConfig {
        &self.cloud_config
    }
}

#[async_trait]
impl Deployment for CloudDeployment {
    async fn new() -> Result<Self, DeploymentError> {
        let cloud_config = CloudConfig::from_env();
        cloud_config
            .apply()
            .map_err(|err| DeploymentError::Other(err.into()))?;
        let inner = LocalDeployment::new().await?;
        let container = CloudContainerService::new(
            inner.local_container_service().clone(),
            inner.secret_store().clone(),
            inner.user_id().to_string(),
            CloudContainerSettings {
                default_image: cloud_config.container_image().to_string(),
                ..CloudContainerSettings::default()
            },
        )
        .await?;
        Ok(Self {
            inner,
            cloud_config,
            container,
        })
    }

    fn user_id(&self) -> &str {
        self.inner.user_id()
    }

    fn shared_types() -> Vec<String> {
        <LocalDeployment as Deployment>::shared_types()
    }

    fn config(&self) -> &Arc<RwLock<Config>> {
        self.inner.config()
    }

    fn db(&self) -> &DBService {
        self.inner.db()
    }

    fn analytics(&self) -> &Option<AnalyticsService> {
        self.inner.analytics()
    }

    fn container(&self) -> &impl ContainerService {
        &self.container
    }

    fn auth(&self) -> &AuthService {
        self.inner.auth()
    }

    fn git(&self) -> &GitService {
        self.inner.git()
    }

    fn image(&self) -> &ImageService {
        self.inner.image()
    }

    fn filesystem(&self) -> &FilesystemService {
        self.inner.filesystem()
    }

    fn msg_stores(&self) -> &Arc<RwLock<HashMap<Uuid, Arc<MsgStore>>>> {
        self.inner.msg_stores()
    }

    fn events(&self) -> &EventService {
        self.inner.events()
    }

    fn file_search_cache(&self) -> &Arc<FileSearchCache> {
        self.inner.file_search_cache()
    }

    fn approvals(&self) -> &Approvals {
        self.inner.approvals()
    }

    fn drafts(&self) -> &DraftsService {
        self.inner.drafts()
    }

    fn secret_store(&self) -> &SecretStore {
        self.inner.secret_store()
    }

    fn claude_auth(&self) -> &ClaudeAuthManager {
        self.inner.claude_auth()
    }
}

impl CloudDeployment {
    pub fn workspace_dir(&self) -> PathBuf {
        self.cloud_config.workspace_dir().clone()
    }
}
