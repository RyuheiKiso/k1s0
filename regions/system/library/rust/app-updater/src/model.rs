use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateType {
    None,
    Optional,
    Mandatory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersionInfo {
    pub latest_version: String,
    pub minimum_version: String,
    pub mandatory: bool,
    pub release_notes: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub minimum_version: String,
    pub update_type: UpdateType,
    pub release_notes: Option<String>,
}

impl UpdateCheckResult {
    pub fn needs_update(&self) -> bool {
        self.update_type != UpdateType::None
    }

    pub fn is_mandatory(&self) -> bool {
        self.update_type == UpdateType::Mandatory
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadArtifactInfo {
    pub url: String,
    pub checksum: String,
    pub size: Option<u64>,
    pub expires_at: Option<DateTime<Utc>>,
}
