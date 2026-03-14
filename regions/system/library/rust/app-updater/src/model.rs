use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// アップデートの種別
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateType {
    /// アップデートなし（現在のバージョンが最新）
    None,
    /// 任意アップデート（新しいバージョンが利用可能）
    Optional,
    /// 強制アップデート（最低バージョンを下回るか mandatory フラグが設定されている）
    Mandatory,
}

/// App Registry サーバーから取得するバージョン情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersionInfo {
    /// 最新バージョン
    pub latest_version: String,
    /// 最低動作バージョン（これを下回る場合は強制アップデート）
    pub minimum_version: String,
    /// 強制アップデートフラグ
    pub mandatory: bool,
    /// リリースノート
    pub release_notes: Option<String>,
    /// リリース日時
    pub published_at: Option<DateTime<Utc>>,
}

/// アップデート確認結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    /// 現在のバージョン
    pub current_version: String,
    /// 最新バージョン
    pub latest_version: String,
    /// 最低動作バージョン
    pub minimum_version: String,
    /// アップデートの種別
    pub update_type: UpdateType,
    /// リリースノート
    pub release_notes: Option<String>,
}

impl UpdateCheckResult {
    /// アップデートが必要かどうかを返す
    pub fn needs_update(&self) -> bool {
        self.update_type != UpdateType::None
    }

    /// 強制アップデートかどうかを返す
    pub fn is_mandatory(&self) -> bool {
        self.update_type == UpdateType::Mandatory
    }
}

/// ダウンロードアーティファクト情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadArtifactInfo {
    /// ダウンロード URL
    pub url: String,
    /// SHA-256 チェックサム（16進数文字列）
    pub checksum: String,
    /// ファイルサイズ（バイト）
    pub size: Option<u64>,
    /// URL の有効期限
    pub expires_at: Option<DateTime<Utc>>,
}
