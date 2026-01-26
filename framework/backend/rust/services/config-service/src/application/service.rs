//! 設定サービス
//!
//! 設定取得のユースケースを実装する。

use std::sync::Arc;

use crate::domain::{ConfigError, Setting, SettingList, SettingQuery, SettingRepository};
use crate::infrastructure::cache::SettingCache;

/// 設定サービス
pub struct ConfigService<R, C>
where
    R: SettingRepository,
    C: SettingCache,
{
    repository: Arc<R>,
    cache: Arc<C>,
    default_env: String,
}

impl<R, C> ConfigService<R, C>
where
    R: SettingRepository,
    C: SettingCache,
{
    /// 新しいサービスを作成
    pub fn new(repository: Arc<R>, cache: Arc<C>, default_env: impl Into<String>) -> Self {
        Self {
            repository,
            cache,
            default_env: default_env.into(),
        }
    }

    /// 設定を取得
    ///
    /// キャッシュを優先し、なければリポジトリから取得してキャッシュに保存する。
    /// 環境が指定されていない場合はデフォルト環境を使用する。
    pub async fn get_setting(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> Result<Setting, ConfigError> {
        let env = env.unwrap_or(&self.default_env);
        let cache_key = format!("{}:{}:{}", service_name, env, key);

        // キャッシュから取得を試みる
        if let Some(setting) = self.cache.get(&cache_key).await {
            return Ok(setting);
        }

        // リポジトリから取得
        let setting = self
            .repository
            .get(service_name, key, Some(env))
            .await?
            .ok_or_else(|| ConfigError::not_found(service_name, key))?;

        // キャッシュに保存
        if let Err(e) = self.cache.set(&cache_key, &setting).await {
            // キャッシュエラーはログのみ、処理は継続
            eprintln!("cache set error: {}", e);
        }

        Ok(setting)
    }

    /// 設定一覧を取得
    pub async fn list_settings(&self, query: &SettingQuery) -> Result<SettingList, ConfigError> {
        self.repository.list(query).await
    }

    /// 設定をリフレッシュ（キャッシュを無効化して再取得）
    pub async fn refresh_setting(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> Result<Setting, ConfigError> {
        let env = env.unwrap_or(&self.default_env);
        let cache_key = format!("{}:{}:{}", service_name, env, key);

        // キャッシュを削除
        let _ = self.cache.delete(&cache_key).await;

        // リポジトリから取得
        let setting = self
            .repository
            .get(service_name, key, Some(env))
            .await?
            .ok_or_else(|| ConfigError::not_found(service_name, key))?;

        // キャッシュに保存
        let _ = self.cache.set(&cache_key, &setting).await;

        Ok(setting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::InMemoryCache;
    use crate::infrastructure::repository::InMemoryRepository;

    #[tokio::test]
    async fn test_get_setting() {
        let repo = Arc::new(InMemoryRepository::new());
        let cache = Arc::new(InMemoryCache::new());

        // 設定を追加
        let setting = Setting::new(1, "test-service", "dev", "feature.enabled", "true");
        repo.save(&setting).await.unwrap();

        let service = ConfigService::new(repo, cache, "dev");

        let result = service
            .get_setting("test-service", "feature.enabled", Some("dev"))
            .await
            .unwrap();

        assert_eq!(result.value, "true");
    }

    #[tokio::test]
    async fn test_get_setting_not_found() {
        let repo = Arc::new(InMemoryRepository::new());
        let cache = Arc::new(InMemoryCache::new());

        let service = ConfigService::new(repo, cache, "dev");

        let result = service
            .get_setting("unknown", "unknown", None)
            .await;

        assert!(matches!(result, Err(ConfigError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_setting_from_cache() {
        let repo = Arc::new(InMemoryRepository::new());
        let cache = Arc::new(InMemoryCache::new());

        // 設定を追加
        let setting = Setting::new(1, "test-service", "dev", "feature.enabled", "true");
        repo.save(&setting).await.unwrap();

        let service = ConfigService::new(Arc::clone(&repo), Arc::clone(&cache), "dev");

        // 1回目の取得（リポジトリから）
        let _ = service
            .get_setting("test-service", "feature.enabled", Some("dev"))
            .await
            .unwrap();

        // リポジトリから設定を削除
        repo.delete("test-service", "feature.enabled", "dev")
            .await
            .unwrap();

        // 2回目の取得（キャッシュから）
        let result = service
            .get_setting("test-service", "feature.enabled", Some("dev"))
            .await
            .unwrap();

        assert_eq!(result.value, "true");
    }

    #[tokio::test]
    async fn test_list_settings() {
        let repo = Arc::new(InMemoryRepository::new());
        let cache = Arc::new(InMemoryCache::new());

        // 設定を追加
        repo.save(&Setting::new(1, "test-service", "dev", "feature.a", "1"))
            .await
            .unwrap();
        repo.save(&Setting::new(2, "test-service", "dev", "feature.b", "2"))
            .await
            .unwrap();

        let service = ConfigService::new(repo, cache, "dev");

        let query = SettingQuery::new()
            .with_service_name("test-service")
            .with_env("dev");
        let result = service.list_settings(&query).await.unwrap();

        assert_eq!(result.settings.len(), 2);
    }
}
