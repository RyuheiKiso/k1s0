//! リポジトリトレイト

use std::future::Future;

use super::entity::{Setting, SettingList, SettingQuery};
use super::error::ConfigError;

/// 設定リポジトリトレイト
pub trait SettingRepository: Send + Sync {
    /// 設定を取得
    fn get(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> impl Future<Output = Result<Option<Setting>, ConfigError>> + Send;

    /// 設定一覧を取得
    fn list(
        &self,
        query: &SettingQuery,
    ) -> impl Future<Output = Result<SettingList, ConfigError>> + Send;

    /// 設定を保存
    fn save(&self, setting: &Setting) -> impl Future<Output = Result<(), ConfigError>> + Send;

    /// 設定を削除
    fn delete(
        &self,
        service_name: &str,
        key: &str,
        env: &str,
    ) -> impl Future<Output = Result<bool, ConfigError>> + Send;
}
