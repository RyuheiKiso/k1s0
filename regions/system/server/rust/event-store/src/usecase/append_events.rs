// イベント追記ユースケース。
// クリーンアーキテクチャの原則により、usecase 層は domain トレイトにのみ依存する。
// 旧実装では execute_with_tx 内で infrastructure 具体型（EventPostgresRepository,
// StreamPostgresRepository）を直接インポートしており C-8 違反であった。
// TransactionalAppendPort を導入してトランザクション処理を抽象化した。

use std::sync::Arc;

use crate::domain::entity::event::{EventData, EventMetadata, EventStream, StoredEvent};
use crate::domain::repository::{EventRepository, EventStreamRepository, TransactionalAppendPort};
use crate::domain::service::{EventStoreDomainError, EventStoreDomainService};

#[derive(Debug, Clone)]
pub struct AppendEventsInput {
    pub stream_id: String,
    /// テナント分離のためのテナント ID（Claims から取得して設定する）
    pub tenant_id: String,
    pub aggregate_type: Option<String>,
    pub events: Vec<EventData>,
    pub expected_version: i64,
}

#[derive(Debug, Clone)]
pub struct AppendEventsOutput {
    pub stream_id: String,
    pub events: Vec<StoredEvent>,
    pub current_version: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum AppendEventsError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    #[error("version conflict for stream {stream_id}: expected {expected}, actual {actual}")]
    VersionConflict {
        stream_id: String,
        expected: i64,
        actual: i64,
    },
    #[error("stream already exists: {0}")]
    StreamAlreadyExists(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// `AppendEventsUseCase` はイベントの追記ユースケースを実装する。
/// `TransactionalAppendPort` が設定されている場合は REPEATABLE READ トランザクションを
/// 使用してストリーム作成・イベント追記・バージョン更新を単一の原子操作として実行する。
/// usecase 層は domain トレイトにのみ依存し、infrastructure 具体型には依存しない。
pub struct AppendEventsUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
    /// トランザクション型追記のドメインポート（PostgreSQL 使用時に設定）
    transactional_port: Option<Arc<dyn TransactionalAppendPort>>,
}

impl AppendEventsUseCase {
    /// リポジトリのみを受け取るコンストラクタ（インメモリ使用時や後方互換用）
    pub fn new(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
    ) -> Self {
        Self {
            stream_repo,
            event_repo,
            transactional_port: None,
        }
    }

    /// `TransactionalAppendPort` を受け取るコンストラクタ（PostgreSQL 使用時はこちらを使用）。
    /// `transactional_port` `が設定されている場合、execute()` は REPEATABLE READ トランザクションで
    /// ストリーム作成・イベント追記・バージョン更新を単一操作として実行する。
    /// domain トレイトを介することで usecase 層の infrastructure 依存を排除する。
    pub fn new_with_transactional_port(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
        transactional_port: Arc<dyn TransactionalAppendPort>,
    ) -> Self {
        Self {
            stream_repo,
            event_repo,
            transactional_port: Some(transactional_port),
        }
    }

    /// 後方互換のために `PgPool` を受け取るコンストラクタを維持する。
    /// 内部で `TransactionalAppendAdapter` を生成して `transactional_port` に設定する。
    #[deprecated(
        since = "0.2.0",
        note = "代わりに new_with_transactional_port を使用してください"
    )]
    #[allow(dead_code)]
    pub fn new_with_pool(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
        pool: sqlx::PgPool,
    ) -> Self {
        use crate::infrastructure::persistence::TransactionalAppendAdapter;
        let port = Arc::new(TransactionalAppendAdapter::new(pool));
        Self {
            stream_repo,
            event_repo,
            transactional_port: Some(port),
        }
    }

    pub async fn execute(
        &self,
        input: &AppendEventsInput,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        if input.events.is_empty() {
            return Err(AppendEventsError::Validation(
                "events must not be empty".to_string(),
            ));
        }

        // TransactionalAppendPort が設定されている場合は単一トランザクションで実行する
        if let Some(port) = &self.transactional_port {
            return self.execute_with_port(input, port.as_ref()).await;
        }

        // インメモリリポジトリ利用時はトランザクションなしで実行する
        self.execute_without_tx(input).await
    }

    /// TransactionalAppendPort（domain トレイト）を介してトランザクション内で実行する。
    /// infrastructure 具体型には依存せず、クリーンアーキテクチャの依存方向を維持する。
    /// `テナント分離のため、tenant_id` を全リポジトリ呼び出しに渡す（ADR-0106）。
    async fn execute_with_port(
        &self,
        input: &AppendEventsInput,
        port: &dyn TransactionalAppendPort,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        // まずトランザクション外でストリームの現在状態を取得してバージョン検証を行う
        // テナント分離のため、tenant_id を渡して RLS を有効化する
        let stream = self
            .stream_repo
            .find_by_id(&input.tenant_id, &input.stream_id)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        // バージョン検証を実行する
        match EventStoreDomainService::validate_append(
            stream.is_some(),
            stream.as_ref().map(|s| s.current_version),
            input.expected_version,
        ) {
            Ok(()) => {}
            Err(EventStoreDomainError::StreamAlreadyExists) => {
                return Err(AppendEventsError::StreamAlreadyExists(
                    input.stream_id.clone(),
                ));
            }
            Err(EventStoreDomainError::StreamNotFound) => {
                return Err(AppendEventsError::StreamNotFound(input.stream_id.clone()));
            }
            Err(EventStoreDomainError::VersionConflict { expected, actual }) => {
                return Err(AppendEventsError::VersionConflict {
                    stream_id: input.stream_id.clone(),
                    expected,
                    actual,
                });
            }
        }

        // 新規ストリームの場合はストリームエンティティを生成する（テナント ID を設定）
        let new_stream = if input.expected_version == -1 {
            Some(EventStream::new(
                input.stream_id.clone(),
                input.aggregate_type.clone().unwrap_or_default(),
                input.tenant_id.clone(),
            ))
        } else {
            None
        };

        let base_version = if input.expected_version == -1 {
            0
        } else {
            input.expected_version
        };

        // 保存するイベントのバージョンを採番する（テナント ID を設定）
        let stored_events: Vec<StoredEvent> = input
            .events
            .iter()
            .enumerate()
            .map(|(i, data)| {
                StoredEvent::new(
                    input.stream_id.clone(),
                    input.tenant_id.clone(),
                    0, // sequence は INSERT の RETURNING で採番される
                    data.event_type.clone(),
                    base_version + (i as i64) + 1,
                    data.payload.clone(),
                    EventMetadata::new(
                        data.metadata.actor_id.clone(),
                        data.metadata.correlation_id.clone(),
                        data.metadata.causation_id.clone(),
                    ),
                )
            })
            .collect();

        let new_version = base_version + input.events.len() as i64;

        // TransactionalAppendPort を介してトランザクション内で原子操作を実行する
        // テナント分離のため tenant_id を渡す（ADR-0106）
        let persisted = port
            .append_in_transaction(
                &input.tenant_id,
                new_stream.as_ref(),
                &input.stream_id,
                stored_events,
                new_version,
            )
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        Ok(AppendEventsOutput {
            stream_id: input.stream_id.clone(),
            events: persisted,
            current_version: new_version,
        })
    }

    /// トランザクションなし（インメモリリポジトリ）で実行する。
    /// `テナント分離のため、tenant_id` を全リポジトリ呼び出しに渡す（ADR-0106）。
    async fn execute_without_tx(
        &self,
        input: &AppendEventsInput,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.tenant_id, &input.stream_id)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        match EventStoreDomainService::validate_append(
            stream.is_some(),
            stream.as_ref().map(|s| s.current_version),
            input.expected_version,
        ) {
            Ok(()) => {}
            Err(EventStoreDomainError::StreamAlreadyExists) => {
                return Err(AppendEventsError::StreamAlreadyExists(
                    input.stream_id.clone(),
                ));
            }
            Err(EventStoreDomainError::StreamNotFound) => {
                return Err(AppendEventsError::StreamNotFound(input.stream_id.clone()));
            }
            Err(EventStoreDomainError::VersionConflict { expected, actual }) => {
                return Err(AppendEventsError::VersionConflict {
                    stream_id: input.stream_id.clone(),
                    expected,
                    actual,
                });
            }
        }

        if input.expected_version == -1 {
            // 新規ストリーム作成時はテナント ID を設定する
            let new_stream = EventStream::new(
                input.stream_id.clone(),
                input.aggregate_type.clone().unwrap_or_default(),
                input.tenant_id.clone(),
            );
            self.stream_repo
                .create(&new_stream)
                .await
                .map_err(|e| AppendEventsError::Internal(e.to_string()))?;
        }

        let base_version = if input.expected_version == -1 {
            0
        } else {
            input.expected_version
        };

        // テナント ID を含むイベントを生成する
        let stored_events: Vec<StoredEvent> = input
            .events
            .iter()
            .enumerate()
            .map(|(i, data)| {
                StoredEvent::new(
                    input.stream_id.clone(),
                    input.tenant_id.clone(),
                    0, // sequence assigned by storage
                    data.event_type.clone(),
                    base_version + (i as i64) + 1,
                    data.payload.clone(),
                    EventMetadata::new(
                        data.metadata.actor_id.clone(),
                        data.metadata.correlation_id.clone(),
                        data.metadata.causation_id.clone(),
                    ),
                )
            })
            .collect();

        let new_version = base_version + input.events.len() as i64;

        // テナント分離のため tenant_id を渡す（ADR-0106）
        let persisted = self
            .event_repo
            .append(&input.tenant_id, &input.stream_id, stored_events)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        self.stream_repo
            .update_version(&input.tenant_id, &input.stream_id, new_version)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        Ok(AppendEventsOutput {
            stream_id: input.stream_id.clone(),
            events: persisted,
            current_version: new_version,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository,
    };

    // テスト用の汎用 AppendEventsInput を生成するファクトリ関数
    fn make_input(stream_id: &str, expected_version: i64) -> AppendEventsInput {
        AppendEventsInput {
            stream_id: stream_id.to_string(),
            tenant_id: "tenant-test".to_string(),
            aggregate_type: Some("TestAggregate".to_string()),
            events: vec![EventData {
                event_type: "TestEventOccurred".to_string(),
                payload: serde_json::json!({"stream_id": stream_id}),
                metadata: EventMetadata::new(Some("user-001".to_string()), None, None),
            }],
            expected_version,
        }
    }

    // テスト用の EventStream を生成するファクトリ関数
    fn make_stream(id: &str, version: i64) -> EventStream {
        EventStream {
            id: id.to_string(),
            tenant_id: "tenant-test".to_string(),
            aggregate_type: "TestAggregate".to_string(),
            current_version: version,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn success_new_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_, _| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo
            .expect_update_version()
            .returning(|_, _, _| Ok(()));

        event_repo.expect_append().returning(|_, stream_id, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    e.sequence = (i as u64) + 1;
                    e.stream_id = stream_id.to_string();
                    e
                })
                .collect())
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-001", -1);
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stream_id, "stream-001");
        assert_eq!(output.current_version, 1);
        assert_eq!(output.events.len(), 1);
    }

    #[tokio::test]
    async fn success_existing_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream("stream-001", 2))));
        stream_repo
            .expect_update_version()
            .returning(|_, _, _| Ok(()));

        event_repo.expect_append().returning(|_, stream_id, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    e.sequence = (i as u64) + 10;
                    e.stream_id = stream_id.to_string();
                    e
                })
                .collect())
        });

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-001", 2);
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.current_version, 3);
    }

    #[tokio::test]
    async fn version_conflict() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream("stream-001", 5))));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-001", 2);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppendEventsError::VersionConflict {
                expected, actual, ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 5);
            }
            e => panic!("unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-999", 0);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::StreamNotFound(_)
        ));
    }

    #[tokio::test]
    async fn stream_already_exists() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream("stream-001", 0))));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-001", -1);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::StreamAlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn validation_empty_events() {
        let stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = AppendEventsInput {
            stream_id: "stream-001".to_string(),
            tenant_id: "tenant-test".to_string(),
            aggregate_type: Some("TestAggregate".to_string()),
            events: vec![],
            expected_version: 0,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppendEventsError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Err(anyhow::anyhow!("db connection failed")));

        let uc = AppendEventsUseCase::new(Arc::new(stream_repo), Arc::new(event_repo));
        let input = make_input("stream-001", 0);
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppendEventsError::Internal(msg) => {
                assert!(msg.contains("db connection failed"));
            }
            e => panic!("unexpected error: {e}"),
        }
    }
}
