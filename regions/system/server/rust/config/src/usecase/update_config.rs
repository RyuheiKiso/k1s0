use std::sync::Arc;

use crate::domain::entity::config_entry::ConfigEntry;
use crate::domain::repository::ConfigRepository;

/// UpdateConfigError は設定値更新に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum UpdateConfigError {
    #[error("config not found: {0}/{1}")]
    NotFound(String, String),

    #[error("version conflict: expected {expected}, current {current}")]
    VersionConflict { expected: i32, current: i32 },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// UpdateConfigInput は設定値更新のリクエストを表す。
#[derive(Debug, Clone)]
pub struct UpdateConfigInput {
    pub namespace: String,
    pub key: String,
    pub value: serde_json::Value,
    pub version: i32,
    pub description: Option<String>,
    pub updated_by: String,
}

/// UpdateConfigUseCase は設定値更新ユースケース。
pub struct UpdateConfigUseCase {
    config_repo: Arc<dyn ConfigRepository>,
}

impl UpdateConfigUseCase {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self { config_repo }
    }

    /// 設定値を更新する（楽観的排他制御付き）。
    pub async fn execute(&self, input: &UpdateConfigInput) -> Result<ConfigEntry, UpdateConfigError> {
        // バリデーション
        if input.namespace.is_empty() {
            return Err(UpdateConfigError::Validation(
                "namespace is required".to_string(),
            ));
        }
        if input.key.is_empty() {
            return Err(UpdateConfigError::Validation(
                "key is required".to_string(),
            ));
        }

        self.config_repo
            .update(
                &input.namespace,
                &input.key,
                &input.value,
                input.version,
                input.description.clone(),
                &input.updated_by,
            )
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    UpdateConfigError::NotFound(input.namespace.clone(), input.key.clone())
                } else if msg.contains("version conflict") {
                    // パースして current version を取得
                    UpdateConfigError::VersionConflict {
                        expected: input.version,
                        current: parse_current_version(&msg).unwrap_or(0),
                    }
                } else {
                    UpdateConfigError::Internal(msg)
                }
            })
    }
}

/// エラーメッセージから current version を取得するヘルパー。
fn parse_current_version(msg: &str) -> Option<i32> {
    // "version conflict: current=4" のような形式を想定
    msg.split("current=")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::config_repository::MockConfigRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_updated_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(50),
            version: 4,
            description: Some("認証サーバーの DB 最大接続数（増設）".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "operator@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_update_input() -> UpdateConfigInput {
        UpdateConfigInput {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(50),
            version: 3,
            description: Some("認証サーバーの DB 最大接続数（増設）".to_string()),
            updated_by: "operator@example.com".to_string(),
        }
    }

    #[tokio::test]
    async fn test_update_config_success() {
        let mut mock = MockConfigRepository::new();
        let updated = make_updated_entry();
        let expected = updated.clone();

        mock.expect_update()
            .withf(|ns, key, _, ver, _, _| {
                ns == "system.auth.database" && key == "max_connections" && *ver == 3
            })
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.value_json, serde_json::json!(50));
        assert_eq!(entry.version, 4);
        assert_eq!(entry.updated_by, expected.updated_by);
    }

    #[tokio::test]
    async fn test_update_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("config not found")));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::NotFound(ns, key) => {
                assert_eq!(ns, "system.auth.database");
                assert_eq!(key, "max_connections");
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_version_conflict() {
        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .returning(|_, _, _, _, _, _| {
                Err(anyhow::anyhow!("version conflict: current=4"))
            });

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::VersionConflict { expected, current } => {
                assert_eq!(expected, 3);
                assert_eq!(current, 4);
            }
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("connection refused")));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_empty_namespace() {
        let mock = MockConfigRepository::new();
        let uc = UpdateConfigUseCase::new(Arc::new(mock));

        let input = UpdateConfigInput {
            namespace: "".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(50),
            version: 3,
            description: None,
            updated_by: "operator@example.com".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::Validation(msg) => assert!(msg.contains("namespace is required")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_empty_key() {
        let mock = MockConfigRepository::new();
        let uc = UpdateConfigUseCase::new(Arc::new(mock));

        let input = UpdateConfigInput {
            namespace: "system.auth.database".to_string(),
            key: "".to_string(),
            value: serde_json::json!(50),
            version: 3,
            description: None,
            updated_by: "operator@example.com".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::Validation(msg) => assert!(msg.contains("key is required")),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_parse_current_version() {
        assert_eq!(parse_current_version("version conflict: current=4"), Some(4));
        assert_eq!(parse_current_version("version conflict: current=10"), Some(10));
        assert_eq!(parse_current_version("no version info"), None);
    }
}
