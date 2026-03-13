use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppUpdaterConfig;
use crate::error::AppUpdaterError;
use crate::model::{AppVersionInfo, UpdateCheckResult};
use crate::version::determine_update_type;

#[async_trait]
pub trait AppUpdater: Send + Sync {
    async fn fetch_version_info(&self) -> Result<AppVersionInfo, AppUpdaterError>;
    async fn check_for_update(&self) -> Result<UpdateCheckResult, AppUpdaterError>;
}

pub struct AppRegistryAppUpdater {
    config: AppUpdaterConfig,
    current_version: String,
}

impl AppRegistryAppUpdater {
    pub fn new(config: AppUpdaterConfig, current_version: String) -> Result<Self, AppUpdaterError> {
        if config.server_url.trim().is_empty() {
            return Err(AppUpdaterError::InvalidConfig(
                "server_url must not be empty".to_string(),
            ));
        }
        if config.app_id.trim().is_empty() {
            return Err(AppUpdaterError::InvalidConfig(
                "app_id must not be empty".to_string(),
            ));
        }
        Ok(Self {
            config,
            current_version,
        })
    }
}

#[async_trait]
impl AppUpdater for AppRegistryAppUpdater {
    async fn fetch_version_info(&self) -> Result<AppVersionInfo, AppUpdaterError> {
        let mut url = format!(
            "{}/apps/{}/versions/latest",
            self.config.server_url.trim_end_matches('/'),
            self.config.app_id,
        );

        let mut params = Vec::new();
        if let Some(ref platform) = self.config.platform {
            params.push(format!("platform={platform}"));
        }
        if let Some(ref arch) = self.config.arch {
            params.push(format!("arch={arch}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let client = reqwest::Client::new();
        let mut builder = client.get(&url);
        if let Some(timeout) = self.config.timeout {
            builder = builder.timeout(timeout);
        }

        let response = builder.send().await.map_err(|e| {
            AppUpdaterError::Connection(e.to_string())
        })?;

        match response.status().as_u16() {
            401 => return Err(AppUpdaterError::Unauthorized),
            404 => {
                return Err(AppUpdaterError::AppNotFound(
                    self.config.app_id.clone(),
                ))
            }
            s if s >= 400 => {
                return Err(AppUpdaterError::Connection(format!(
                    "server returned status {s}"
                )))
            }
            _ => {}
        }

        let info: AppVersionInfo = response
            .json()
            .await
            .map_err(|e| AppUpdaterError::Parse(e.to_string()))?;

        Ok(info)
    }

    async fn check_for_update(&self) -> Result<UpdateCheckResult, AppUpdaterError> {
        let version_info = self.fetch_version_info().await?;
        let update_type = determine_update_type(&self.current_version, &version_info);

        Ok(UpdateCheckResult {
            current_version: self.current_version.clone(),
            latest_version: version_info.latest_version.clone(),
            minimum_version: version_info.minimum_version.clone(),
            update_type,
            release_notes: version_info.release_notes.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// InMemoryAppUpdater
// ---------------------------------------------------------------------------

struct InMemoryState {
    version_info: AppVersionInfo,
    current_version: String,
}

pub struct InMemoryAppUpdater {
    state: Arc<RwLock<InMemoryState>>,
}

impl InMemoryAppUpdater {
    pub fn new(version_info: AppVersionInfo, current_version: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(InMemoryState {
                version_info,
                current_version,
            })),
        }
    }

    pub async fn set_version_info(&self, info: AppVersionInfo) {
        self.state.write().await.version_info = info;
    }

    pub async fn set_current_version(&self, version: String) {
        self.state.write().await.current_version = version;
    }
}

impl Default for InMemoryAppUpdater {
    fn default() -> Self {
        Self::new(
            AppVersionInfo {
                latest_version: "0.0.0".to_string(),
                minimum_version: "0.0.0".to_string(),
                mandatory: false,
                release_notes: None,
                published_at: None,
            },
            "0.0.0".to_string(),
        )
    }
}

#[async_trait]
impl AppUpdater for InMemoryAppUpdater {
    async fn fetch_version_info(&self) -> Result<AppVersionInfo, AppUpdaterError> {
        Ok(self.state.read().await.version_info.clone())
    }

    async fn check_for_update(&self) -> Result<UpdateCheckResult, AppUpdaterError> {
        let state = self.state.read().await;
        let update_type = determine_update_type(&state.current_version, &state.version_info);

        Ok(UpdateCheckResult {
            current_version: state.current_version.clone(),
            latest_version: state.version_info.latest_version.clone(),
            minimum_version: state.version_info.minimum_version.clone(),
            update_type,
            release_notes: state.version_info.release_notes.clone(),
        })
    }
}
