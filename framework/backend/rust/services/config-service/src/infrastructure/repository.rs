//! リポジトリ実装

use std::collections::HashMap;
use std::sync::RwLock;

use crate::domain::{ConfigError, Setting, SettingList, SettingQuery, SettingRepository};

/// インメモリリポジトリ
///
/// 開発/テスト用の簡易実装。
pub struct InMemoryRepository {
    // key: "service_name:env:key"
    settings: RwLock<HashMap<String, Setting>>,
}

impl InMemoryRepository {
    /// 新しいリポジトリを作成
    pub fn new() -> Self {
        Self {
            settings: RwLock::new(HashMap::new()),
        }
    }

    /// キーを生成
    fn make_key(service_name: &str, env: &str, key: &str) -> String {
        format!("{}:{}:{}", service_name, env, key)
    }

    /// 設定数を取得
    pub fn len(&self) -> usize {
        self.settings.read().unwrap().len()
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.settings.read().unwrap().is_empty()
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingRepository for InMemoryRepository {
    async fn get(
        &self,
        service_name: &str,
        key: &str,
        env: Option<&str>,
    ) -> Result<Option<Setting>, ConfigError> {
        let settings = self.settings.read().unwrap();

        // 指定された環境で検索
        if let Some(env) = env {
            let storage_key = Self::make_key(service_name, env, key);
            if let Some(setting) = settings.get(&storage_key) {
                if setting.is_active {
                    return Ok(Some(setting.clone()));
                }
            }
        }

        // デフォルト環境で検索
        let default_key = Self::make_key(service_name, "default", key);
        if let Some(setting) = settings.get(&default_key) {
            if setting.is_active {
                return Ok(Some(setting.clone()));
            }
        }

        Ok(None)
    }

    async fn list(&self, query: &SettingQuery) -> Result<SettingList, ConfigError> {
        let settings = self.settings.read().unwrap();

        let mut results: Vec<Setting> = settings
            .values()
            .filter(|s| {
                // サービス名フィルタ
                if let Some(ref service_name) = query.service_name {
                    if &s.service_name != service_name {
                        return false;
                    }
                }

                // 環境フィルタ
                if let Some(ref env) = query.env {
                    if &s.env != env {
                        return false;
                    }
                }

                // キープレフィックスフィルタ
                if let Some(ref prefix) = query.key_prefix {
                    if !s.key.starts_with(prefix) {
                        return false;
                    }
                }

                // アクティブのみ
                s.is_active
            })
            .cloned()
            .collect();

        // キーでソート
        results.sort_by(|a, b| a.key.cmp(&b.key));

        // ページネーション
        let page_size = query.page_size.unwrap_or(100).min(1000) as usize;

        // ページトークンがある場合はスキップ
        let start_index = if let Some(ref token) = query.page_token {
            // トークンは "offset:{n}" 形式
            token
                .strip_prefix("offset:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        let total = results.len();
        let end_index = (start_index + page_size).min(total);
        let page_results = results[start_index..end_index].to_vec();

        let next_page_token = if end_index < total {
            Some(format!("offset:{}", end_index))
        } else {
            None
        };

        let mut list = SettingList::new(page_results);
        if let Some(token) = next_page_token {
            list = list.with_next_page_token(token);
        }

        Ok(list)
    }

    async fn save(&self, setting: &Setting) -> Result<(), ConfigError> {
        let key = Self::make_key(&setting.service_name, &setting.env, &setting.key);
        let mut settings = self.settings.write().unwrap();
        settings.insert(key, setting.clone());
        Ok(())
    }

    async fn delete(
        &self,
        service_name: &str,
        key: &str,
        env: &str,
    ) -> Result<bool, ConfigError> {
        let storage_key = Self::make_key(service_name, env, key);
        let mut settings = self.settings.write().unwrap();
        Ok(settings.remove(&storage_key).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_get() {
        let repo = InMemoryRepository::new();

        let setting = Setting::new(1, "my-service", "dev", "feature.enabled", "true");
        repo.save(&setting).await.unwrap();

        let result = repo.get("my-service", "feature.enabled", Some("dev")).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "true");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let repo = InMemoryRepository::new();

        let result = repo.get("unknown", "unknown", Some("dev")).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_fallback_to_default() {
        let repo = InMemoryRepository::new();

        // デフォルト環境に設定を追加
        let setting = Setting::new(1, "my-service", "default", "feature.enabled", "true");
        repo.save(&setting).await.unwrap();

        // prodで検索してもdefaultにフォールバック
        let result = repo.get("my-service", "feature.enabled", Some("prod")).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "true");
    }

    #[tokio::test]
    async fn test_list_with_filter() {
        let repo = InMemoryRepository::new();

        repo.save(&Setting::new(1, "service-a", "dev", "key1", "1"))
            .await
            .unwrap();
        repo.save(&Setting::new(2, "service-a", "dev", "key2", "2"))
            .await
            .unwrap();
        repo.save(&Setting::new(3, "service-b", "dev", "key1", "3"))
            .await
            .unwrap();

        let query = SettingQuery::new().with_service_name("service-a");
        let result = repo.list(&query).await.unwrap();
        assert_eq!(result.settings.len(), 2);
    }

    #[tokio::test]
    async fn test_list_with_prefix() {
        let repo = InMemoryRepository::new();

        repo.save(&Setting::new(1, "svc", "dev", "feature.a", "1"))
            .await
            .unwrap();
        repo.save(&Setting::new(2, "svc", "dev", "feature.b", "2"))
            .await
            .unwrap();
        repo.save(&Setting::new(3, "svc", "dev", "other.c", "3"))
            .await
            .unwrap();

        let query = SettingQuery::new()
            .with_service_name("svc")
            .with_key_prefix("feature.");
        let result = repo.list(&query).await.unwrap();
        assert_eq!(result.settings.len(), 2);
    }

    #[tokio::test]
    async fn test_list_pagination() {
        let repo = InMemoryRepository::new();

        for i in 1..=5 {
            repo.save(&Setting::new(i, "svc", "dev", format!("key{}", i), format!("{}", i)))
                .await
                .unwrap();
        }

        let query = SettingQuery::new()
            .with_service_name("svc")
            .with_page_size(2);
        let result = repo.list(&query).await.unwrap();

        assert_eq!(result.settings.len(), 2);
        assert!(result.next_page_token.is_some());

        // 次ページ
        let query = SettingQuery::new()
            .with_service_name("svc")
            .with_page_size(2)
            .with_page_token(result.next_page_token.unwrap());
        let result = repo.list(&query).await.unwrap();

        assert_eq!(result.settings.len(), 2);
        assert!(result.next_page_token.is_some());
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = InMemoryRepository::new();

        let setting = Setting::new(1, "svc", "dev", "key", "value");
        repo.save(&setting).await.unwrap();
        assert_eq!(repo.len(), 1);

        let deleted = repo.delete("svc", "key", "dev").await.unwrap();
        assert!(deleted);
        assert!(repo.is_empty());
    }
}
