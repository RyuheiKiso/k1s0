//! 設定サービス
//!
//! 設定取得のユースケースを実装する。

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::domain::{ConfigError, Setting, SettingList, SettingQuery, SettingRepository};
use crate::infrastructure::cache::SettingCache;

/// 設定変更イベント
#[derive(Debug, Clone)]
pub enum SettingChangeEvent {
    /// 設定が更新された
    Updated(Setting),
    /// 設定が削除された
    Deleted {
        service_name: String,
        env: String,
        key: String,
    },
}

/// 設定サービス
pub struct ConfigService<R, C>
where
    R: SettingRepository,
    C: SettingCache,
{
    repository: Arc<R>,
    cache: Arc<C>,
    default_env: String,
    /// 設定変更通知チャネル
    change_sender: broadcast::Sender<SettingChangeEvent>,
}

impl<R, C> ConfigService<R, C>
where
    R: SettingRepository,
    C: SettingCache,
{
    /// 新しいサービスを作成
    pub fn new(repository: Arc<R>, cache: Arc<C>, default_env: impl Into<String>) -> Self {
        let (change_sender, _) = broadcast::channel(256);
        Self {
            repository,
            cache,
            default_env: default_env.into(),
            change_sender,
        }
    }

    /// 設定変更を購読
    pub fn subscribe(&self) -> broadcast::Receiver<SettingChangeEvent> {
        self.change_sender.subscribe()
    }

    /// 設定を更新（変更通知付き）
    pub async fn update_setting(&self, setting: &Setting) -> Result<(), ConfigError> {
        // リポジトリに保存
        self.repository.save(setting).await?;

        // キャッシュを更新
        let cache_key = format!("{}:{}:{}", setting.service_name, setting.env, setting.key);
        let _ = self.cache.set(&cache_key, setting).await;

        // 変更を通知
        let _ = self.change_sender.send(SettingChangeEvent::Updated(setting.clone()));

        Ok(())
    }

    /// 設定を削除（変更通知付き）
    pub async fn delete_setting(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> Result<(), ConfigError> {
        let env = env.unwrap_or(&self.default_env);

        // リポジトリから削除
        self.repository.delete(service_name, key, env).await?;

        // キャッシュから削除
        let cache_key = format!("{}:{}:{}", service_name, env, key);
        let _ = self.cache.delete(&cache_key).await;

        // 変更を通知
        let _ = self.change_sender.send(SettingChangeEvent::Deleted {
            service_name: service_name.to_string(),
            env: env.to_string(),
            key: key.to_string(),
        });

        Ok(())
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
