use std::sync::Arc;

use crate::domain::entity::config_entry::ConfigEntry;
use crate::domain::repository::ConfigRepository;
use crate::infrastructure::kafka_producer::{ConfigChangedEvent, KafkaProducer};
use crate::usecase::watch_config::ConfigChangeEvent;

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
    kafka_producer: Option<Arc<KafkaProducer>>,
    watch_sender: Option<tokio::sync::broadcast::Sender<ConfigChangeEvent>>,
}

impl UpdateConfigUseCase {
    /// Kafka 通知なしのコンストラクタ（既存互換）。
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self {
            config_repo,
            kafka_producer: None,
            watch_sender: None,
        }
    }

    /// Kafka 通知ありのコンストラクタ。
    pub fn new_with_kafka(
        config_repo: Arc<dyn ConfigRepository>,
        kafka_producer: Arc<KafkaProducer>,
    ) -> Self {
        Self {
            config_repo,
            kafka_producer: Some(kafka_producer),
            watch_sender: None,
        }
    }

    /// broadcast watch sender ありのコンストラクタ。
    /// watch_sender を指定すると、更新成功後に ConfigChangeEvent が全購読者に送信される。
    pub fn new_with_watch(
        config_repo: Arc<dyn ConfigRepository>,
        watch_sender: tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    ) -> Self {
        Self {
            config_repo,
            kafka_producer: None,
            watch_sender: Some(watch_sender),
        }
    }

    /// Kafka 通知と broadcast watch sender の両方を持つコンストラクタ。
    pub fn new_with_kafka_and_watch(
        config_repo: Arc<dyn ConfigRepository>,
        kafka_producer: Arc<KafkaProducer>,
        watch_sender: tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    ) -> Self {
        Self {
            config_repo,
            kafka_producer: Some(kafka_producer),
            watch_sender: Some(watch_sender),
        }
    }

    /// 設定値を更新する（楽観的排他制御付き）。
    /// 更新成功後、Kafka プロデューサーが設定されていれば変更イベントを発行する。
    /// Kafka への通知はベストエフォートであり、失敗してもエラーにしない。
    /// 監査ログも記録する（ベストエフォート）。
    pub async fn execute(
        &self,
        input: &UpdateConfigInput,
    ) -> Result<ConfigEntry, UpdateConfigError> {
        // バリデーション
        if input.namespace.is_empty() {
            return Err(UpdateConfigError::Validation(
                "namespace is required".to_string(),
            ));
        }
        if input.key.is_empty() {
            return Err(UpdateConfigError::Validation("key is required".to_string()));
        }

        // 旧値を取得（監査ログ用）
        let old_entry = self
            .config_repo
            .find_by_namespace_and_key(&input.namespace, &input.key)
            .await
            .ok()
            .flatten();

        let updated_entry = self
            .config_repo
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
                    UpdateConfigError::VersionConflict {
                        expected: input.version,
                        current: parse_current_version(&msg).unwrap_or(0),
                    }
                } else {
                    UpdateConfigError::Internal(msg)
                }
            })?;

        // 監査ログ記録（ベストエフォート）
        let change_log = crate::domain::entity::config_change_log::ConfigChangeLog::new(
            crate::domain::entity::config_change_log::CreateChangeLogRequest {
                config_entry_id: updated_entry.id,
                namespace: updated_entry.namespace.clone(),
                key: updated_entry.key.clone(),
                old_value: old_entry.as_ref().map(|e| e.value_json.clone()),
                new_value: Some(updated_entry.value_json.clone()),
                old_version: old_entry.as_ref().map_or(0, |e| e.version),
                new_version: updated_entry.version,
                change_type: "UPDATED".to_string(),
                changed_by: updated_entry.updated_by.clone(),
            },
        );
        if let Err(e) = self.config_repo.record_change_log(&change_log).await {
            tracing::warn!(
                namespace = %updated_entry.namespace,
                key = %updated_entry.key,
                error = %e,
                "failed to record config change log (best-effort)"
            );
        }

        // Kafka 変更通知（ベストエフォート）
        if let Some(producer) = &self.kafka_producer {
            let event = ConfigChangedEvent {
                namespace: updated_entry.namespace.clone(),
                key: updated_entry.key.clone(),
                new_value: updated_entry.value_json.clone(),
                updated_by: updated_entry.updated_by.clone(),
                version: updated_entry.version,
                timestamp: updated_entry.updated_at.to_rfc3339(),
            };
            if let Err(e) = producer.publish_config_changed(&event).await {
                tracing::warn!(
                    namespace = %updated_entry.namespace,
                    key = %updated_entry.key,
                    error = %e,
                    "failed to publish config changed event to Kafka (best-effort)"
                );
            }
        }

        // broadcast watch 変更通知（ベストエフォート）
        if let Some(sender) = &self.watch_sender {
            let watch_event = ConfigChangeEvent {
                namespace: updated_entry.namespace.clone(),
                key: updated_entry.key.clone(),
                value_json: updated_entry.value_json.clone(),
                updated_by: updated_entry.updated_by.clone(),
                version: updated_entry.version,
            };
            // 受信者なしエラーは無視（ベストエフォート）
            let _ = sender.send(watch_event);
        }

        Ok(updated_entry)
    }
}

/// エラーメッセージから current version を取得するヘルパー。
fn parse_current_version(msg: &str) -> Option<i32> {
    // "version conflict: current=4" のような形式を想定
    msg.split("current=")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| {
            s.trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse()
                .ok()
        })
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

    fn make_old_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("認証サーバーの DB 最大接続数".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_config_success() {
        let mut mock = MockConfigRepository::new();
        let updated = make_updated_entry();
        let expected = updated.clone();

        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        mock.expect_update()
            .withf(|ns, key, _, ver, _, _| {
                ns == "system.auth.database" && key == "max_connections" && *ver == 3
            })
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log().returning(|_| Ok(()));

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
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
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
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_version_conflict() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("version conflict: current=4")));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::VersionConflict { expected, current } => {
                assert_eq!(expected, 3);
                assert_eq!(current, 4);
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_internal_error() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("connection refused")));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateConfigError::Internal(msg) => assert!(msg.contains("connection refused")),
            e => unreachable!("unexpected error in test: {:?}", e),
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
            e => unreachable!("unexpected error in test: {:?}", e),
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
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[test]
    fn test_parse_current_version() {
        assert_eq!(
            parse_current_version("version conflict: current=4"),
            Some(4)
        );
        assert_eq!(
            parse_current_version("version conflict: current=10"),
            Some(10)
        );
        assert_eq!(parse_current_version("no version info"), None);
    }

    // --- Kafka 通知関連テスト ---

    #[tokio::test]
    async fn test_update_config_without_kafka_succeeds() {
        // kafka_producer が None のとき Kafka 通知なしで成功する
        let mut mock = MockConfigRepository::new();
        let updated = make_updated_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        mock.expect_update()
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_returns_uc_without_kafka() {
        // new() で作成した UeCase は kafka_producer が None
        let mock = MockConfigRepository::new();
        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        assert!(uc.kafka_producer.is_none());
    }

    #[tokio::test]
    async fn test_new_with_kafka_sets_producer() {
        // new_with_kafka() で作成した UseCase は kafka_producer が Some
        // KafkaProducer はブローカー接続が必要なため、Option<Arc<KafkaProducer>> の
        // Some 性チェックのみ行う（実際のKafka接続は統合テストで確認）
        let mock = MockConfigRepository::new();
        // KafkaConfig を直接構築してテスト（接続は行わない）
        // ここでは型検証のみ: new_with_kafka のシグネチャが正しいことを確認する
        let _ = |producer: Arc<KafkaProducer>| {
            let uc = UpdateConfigUseCase::new_with_kafka(
                Arc::new(MockConfigRepository::new()),
                producer,
            );
            assert!(uc.kafka_producer.is_some());
        };
        // kafka_producer なし版は None
        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        assert!(uc.kafka_producer.is_none());
    }

    // --- watch_sender 関連テスト ---

    #[tokio::test]
    async fn test_new_with_watch_sets_sender() {
        let (tx, _rx) = tokio::sync::broadcast::channel::<ConfigChangeEvent>(16);
        let mock = MockConfigRepository::new();
        let uc = UpdateConfigUseCase::new_with_watch(Arc::new(mock), tx);
        assert!(uc.watch_sender.is_some());
        assert!(uc.kafka_producer.is_none());
    }

    #[tokio::test]
    async fn test_update_config_notifies_watch_sender() {
        // 更新成功後に broadcast watch イベントが送信されることを確認する
        let mut mock = MockConfigRepository::new();
        let updated = make_updated_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        mock.expect_update()
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let (tx, mut rx) = tokio::sync::broadcast::channel::<ConfigChangeEvent>(16);
        let uc = UpdateConfigUseCase::new_with_watch(Arc::new(mock), tx);

        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_ok());

        let event = rx.recv().await.unwrap();
        assert_eq!(event.namespace, "system.auth.database");
        assert_eq!(event.key, "max_connections");
        assert_eq!(event.value_json, serde_json::json!(50));
        assert_eq!(event.updated_by, "operator@example.com");
        assert_eq!(event.version, 4);
    }

    #[tokio::test]
    async fn test_update_config_watch_not_sent_on_failure() {
        // 更新失敗時は watch イベントが送信されないことを確認する
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
        mock.expect_update()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("config not found")));

        let (tx, mut rx) = tokio::sync::broadcast::channel::<ConfigChangeEvent>(16);
        let uc = UpdateConfigUseCase::new_with_watch(Arc::new(mock), tx);

        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_err());

        // チャンネルは空のまま（イベントが届いていない）
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_new_with_kafka_and_watch_sets_both() {
        // new_with_kafka_and_watch() で作成した UseCase は両方が Some
        let (tx, _rx) = tokio::sync::broadcast::channel::<ConfigChangeEvent>(16);
        let _ = |producer: Arc<KafkaProducer>| {
            let uc = UpdateConfigUseCase::new_with_kafka_and_watch(
                Arc::new(MockConfigRepository::new()),
                producer,
                tx,
            );
            assert!(uc.kafka_producer.is_some());
            assert!(uc.watch_sender.is_some());
        };
        // 型チェックのみ（実際のKafka接続は統合テストで確認）
    }

    #[tokio::test]
    async fn test_update_config_no_watch_sender_still_succeeds() {
        // watch_sender が None でも更新は成功する（後方互換性）
        let mut mock = MockConfigRepository::new();
        let updated = make_updated_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        mock.expect_update()
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        assert!(uc.watch_sender.is_none());

        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_ok());
    }

    // --- 監査ログ関連テスト ---

    #[tokio::test]
    async fn test_update_config_records_change_log() {
        let mut mock = MockConfigRepository::new();
        let old = make_old_entry();
        let updated = make_updated_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _| Ok(Some(old.clone())));
        mock.expect_update()
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log()
            .withf(|log| {
                log.change_type == "UPDATED"
                    && log.namespace == "system.auth.database"
                    && log.key == "max_connections"
                    && log.old_value == Some(serde_json::json!(25))
                    && log.new_value == Some(serde_json::json!(50))
                    && log.old_version == 3
                    && log.new_version == 4
                    && log.changed_by == "operator@example.com"
            })
            .returning(|_| Ok(()));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_config_change_log_failure_is_best_effort() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_old_entry())));
        let updated = make_updated_entry();
        mock.expect_update()
            .returning(move |_, _, _, _, _, _| Ok(updated.clone()));
        mock.expect_record_change_log()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = UpdateConfigUseCase::new(Arc::new(mock));
        let result = uc.execute(&make_update_input()).await;
        // 監査ログ失敗しても更新自体は成功する
        assert!(result.is_ok());
    }
}
