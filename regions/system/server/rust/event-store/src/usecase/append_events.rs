use std::sync::Arc;

use crate::domain::entity::event::{EventData, EventMetadata, EventStream, StoredEvent};
use crate::domain::repository::{EventRepository, EventStreamRepository};
use crate::domain::service::{EventStoreDomainError, EventStoreDomainService};

#[derive(Debug, Clone)]
pub struct AppendEventsInput {
    pub stream_id: String,
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

/// AppendEventsUseCase はイベントの追記ユースケースを実装する。
/// PgPool が存在する場合は REPEATABLE READ トランザクションを使用して
/// ストリーム作成・イベント追記・バージョン更新を単一の原子操作として実行する。
pub struct AppendEventsUseCase {
    stream_repo: Arc<dyn EventStreamRepository>,
    event_repo: Arc<dyn EventRepository>,
    /// DB トランザクション管理用のコネクションプール（PostgreSQL 使用時に設定）
    pool: Option<sqlx::PgPool>,
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
            pool: None,
        }
    }

    /// PgPool を受け取るコンストラクタ（PostgreSQL 使用時はこちらを使用）。
    /// pool が設定されている場合、execute() は REPEATABLE READ トランザクションで
    /// ストリーム作成・イベント追記・バージョン更新を単一操作として実行する。
    pub fn new_with_pool(
        stream_repo: Arc<dyn EventStreamRepository>,
        event_repo: Arc<dyn EventRepository>,
        pool: sqlx::PgPool,
    ) -> Self {
        Self {
            stream_repo,
            event_repo,
            pool: Some(pool),
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

        // PgPool がある場合は単一 REPEATABLE READ トランザクションで実行する
        if let Some(pool) = &self.pool {
            return self.execute_with_tx(input, pool).await;
        }

        // インメモリリポジトリ利用時はトランザクションなしで実行する
        self.execute_without_tx(input).await
    }

    /// REPEATABLE READ トランザクション内でストリーム作成・イベント追記・バージョン更新を実行する。
    /// ファントムリードを防止し、バージョンチェックから書き込みまでの一貫性を保証する。
    async fn execute_with_tx(
        &self,
        input: &AppendEventsInput,
        pool: &sqlx::PgPool,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        use crate::infrastructure::persistence::{EventPostgresRepository, StreamPostgresRepository};
        use sqlx::Executor;

        // REPEATABLE READ でトランザクションを開始してファントムリードを防止する
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;
        // トランザクション分離レベルを REPEATABLE READ に設定してファントムリードを防止する
        tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        // トランザクション内でストリームの現在状態を取得する
        let stream = sqlx::query_as::<_, (String, String, i64, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT id, aggregate_type, current_version, created_at, updated_at
               FROM eventstore.event_streams WHERE id = $1"#,
        )
        .bind(&input.stream_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppendEventsError::Internal(e.to_string()))?
        .map(|(id, aggregate_type, current_version, created_at, updated_at)| EventStream {
            id,
            aggregate_type,
            current_version,
            created_at,
            updated_at,
        });

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

        // 新規ストリームの場合はトランザクション内でストリームを作成する
        if input.expected_version == -1 {
            let new_stream = EventStream::new(
                input.stream_id.clone(),
                input.aggregate_type.clone().unwrap_or_default(),
            );
            StreamPostgresRepository::create_in_tx(&new_stream, &mut tx)
                .await
                .map_err(|e| AppendEventsError::Internal(e.to_string()))?;
        }

        let base_version = if input.expected_version == -1 {
            0
        } else {
            input.expected_version
        };

        // 保存するイベントのバージョンを採番する
        let stored_events: Vec<StoredEvent> = input
            .events
            .iter()
            .enumerate()
            .map(|(i, data)| {
                StoredEvent::new(
                    input.stream_id.clone(),
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

        // トランザクション内でイベントを一括INSERTする
        let persisted =
            EventPostgresRepository::append_in_tx(&input.stream_id, stored_events, &mut tx)
                .await
                .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        // トランザクション内でストリームのバージョンを更新する
        StreamPostgresRepository::update_version_in_tx(&input.stream_id, new_version, &mut tx)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        // 全操作成功後にコミットして原子性を保証する
        tx.commit()
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        Ok(AppendEventsOutput {
            stream_id: input.stream_id.clone(),
            events: persisted,
            current_version: new_version,
        })
    }

    /// トランザクションなし（インメモリリポジトリ）で実行する。
    async fn execute_without_tx(
        &self,
        input: &AppendEventsInput,
    ) -> Result<AppendEventsOutput, AppendEventsError> {
        let stream = self
            .stream_repo
            .find_by_id(&input.stream_id)
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
            let new_stream = EventStream::new(
                input.stream_id.clone(),
                input.aggregate_type.clone().unwrap_or_default(),
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

        let stored_events: Vec<StoredEvent> = input
            .events
            .iter()
            .enumerate()
            .map(|(i, data)| {
                StoredEvent::new(
                    input.stream_id.clone(),
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

        let persisted = self
            .event_repo
            .append(&input.stream_id, stored_events)
            .await
            .map_err(|e| AppendEventsError::Internal(e.to_string()))?;

        self.stream_repo
            .update_version(&input.stream_id, new_version)
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
            aggregate_type: Some("TestAggregate".to_string()),
            events: vec![EventData {
                event_type: "TestEventOccurred".to_string(),
                payload: serde_json::json!({"stream_id": stream_id}),
                metadata: EventMetadata::new(Some("user-001".to_string()), None, None),
            }],
            expected_version,
        }
    }

    #[tokio::test]
    async fn success_new_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();

        stream_repo.expect_find_by_id().returning(|_| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo.expect_update_version().returning(|_, _| Ok(()));

        event_repo.expect_append().returning(|stream_id, events| {
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

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "stream-001".to_string(),
                aggregate_type: "TestAggregate".to_string(),
                current_version: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });
        stream_repo.expect_update_version().returning(|_, _| Ok(()));

        event_repo.expect_append().returning(|stream_id, events| {
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

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "stream-001".to_string(),
                aggregate_type: "TestAggregate".to_string(),
                current_version: 5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

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

        stream_repo.expect_find_by_id().returning(|_| Ok(None));

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

        stream_repo.expect_find_by_id().returning(|_| {
            Ok(Some(EventStream {
                id: "stream-001".to_string(),
                aggregate_type: "TestAggregate".to_string(),
                current_version: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

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
            .returning(|_| Err(anyhow::anyhow!("db connection failed")));

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
