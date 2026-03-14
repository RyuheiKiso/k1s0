//! アプリアップデーターライブラリ
//!
//! アプリケーションのバージョン管理とアップデート確認機能を提供する。
//! App Registry サーバーと通信してバージョン情報を取得し、
//! 現在のバージョンと最新バージョンを比較してアップデートの要否を判定する。

pub mod checksum;
pub mod client;
pub mod config;
pub mod error;
pub mod model;
pub mod version;

pub use checksum::ChecksumVerifier;
pub use client::{AppRegistryAppUpdater, AppUpdater, InMemoryAppUpdater};
pub use config::AppUpdaterConfig;
pub use error::AppUpdaterError;
pub use model::{AppVersionInfo, DownloadArtifactInfo, UpdateCheckResult, UpdateType};
pub use version::{compare_versions, determine_update_type};
