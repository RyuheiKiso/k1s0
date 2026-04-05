use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::adapter::handler::event_handler::spawn_publish_events_with_retry;
use crate::domain::entity::event::{EventData, EventMetadata};
use crate::domain::repository::EventStreamRepository;
use crate::infrastructure::kafka::EventPublisher;
use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::common::v1::Timestamp as ProtoTimestamp;
use crate::proto::k1s0::system::eventstore::v1::{
    AppendEventsRequest as ProtoAppendEventsRequest,
    AppendEventsResponse as ProtoAppendEventsResponse,
    CreateSnapshotRequest as ProtoCreateSnapshotRequest,
    CreateSnapshotResponse as ProtoCreateSnapshotResponse,
    DeleteStreamRequest as ProtoDeleteStreamRequest,
    DeleteStreamResponse as ProtoDeleteStreamResponse, EventStoreMetadata as ProtoEventMetadata,
    GetLatestSnapshotRequest as ProtoGetLatestSnapshotRequest,
    GetLatestSnapshotResponse as ProtoGetLatestSnapshotResponse,
    ListStreamsRequest as ProtoListStreamsRequest, ListStreamsResponse as ProtoListStreamsResponse,
    ReadEventBySequenceRequest as ProtoReadEventBySequenceRequest,
    ReadEventBySequenceResponse as ProtoReadEventBySequenceResponse,
    ReadEventsRequest as ProtoReadEventsRequest, ReadEventsResponse as ProtoReadEventsResponse,
    Snapshot as ProtoSnapshot, StoredEvent as ProtoStoredEvent, StreamInfo as ProtoStreamInfo,
};
use crate::usecase::append_events::{AppendEventsError, AppendEventsInput, AppendEventsUseCase};
use crate::usecase::create_snapshot::{
    CreateSnapshotError, CreateSnapshotInput, CreateSnapshotUseCase,
};
use crate::usecase::delete_stream::{DeleteStreamError, DeleteStreamInput, DeleteStreamUseCase};
use crate::usecase::get_latest_snapshot::{
    GetLatestSnapshotError, GetLatestSnapshotInput, GetLatestSnapshotUseCase,
};
use crate::usecase::read_event_by_sequence::{
    ReadEventBySequenceError, ReadEventBySequenceInput, ReadEventBySequenceUseCase,
};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput, ReadEventsUseCase};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("aborted: {0}")]
    Aborted(String),

    #[error("internal: {0}")]
    Internal(String),
}

/// gRPC サービスの実装。各ユースケースとリポジトリ、Kafka パブリッシャーを保持する。
pub struct EventStoreGrpcService {
    append_events_uc: Arc<AppendEventsUseCase>,
    read_events_uc: Arc<ReadEventsUseCase>,
    read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
    create_snapshot_uc: Arc<CreateSnapshotUseCase>,
    get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
    delete_stream_uc: Arc<DeleteStreamUseCase>,
    stream_repo: Arc<dyn EventStreamRepository>,
    /// Kafka イベントパブリッシャー。REST と同様に append 後にイベントを発行する。
    event_publisher: Arc<dyn EventPublisher>,
}

impl EventStoreGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        append_events_uc: Arc<AppendEventsUseCase>,
        read_events_uc: Arc<ReadEventsUseCase>,
        read_event_by_sequence_uc: Arc<ReadEventBySequenceUseCase>,
        create_snapshot_uc: Arc<CreateSnapshotUseCase>,
        get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
        delete_stream_uc: Arc<DeleteStreamUseCase>,
        stream_repo: Arc<dyn EventStreamRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            append_events_uc,
            read_events_uc,
            read_event_by_sequence_uc,
            create_snapshot_uc,
            get_latest_snapshot_uc,
            delete_stream_uc,
            stream_repo,
            event_publisher,
        }
    }

    pub async fn list_streams(
        &self,
        req: ProtoListStreamsRequest,
    ) -> Result<ProtoListStreamsResponse, GrpcError> {
        let pagination = req.pagination.unwrap_or_default();
        let page = if pagination.page <= 0 {
            1
        } else {
            pagination.page as u32
        };
        let page_size = if pagination.page_size <= 0 {
            50
        } else {
            pagination.page_size as u32
        };

        let (streams, total_count) = self
            .stream_repo
            .list_all(page, page_size)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        let has_next = (page as u64) * (page_size as u64) < total_count;

        Ok(ProtoListStreamsResponse {
            streams: streams
                .into_iter()
                .map(|stream| ProtoStreamInfo {
                    id: stream.id,
                    aggregate_type: stream.aggregate_type,
                    current_version: stream.current_version,
                    created_at: datetime_to_proto_timestamp(stream.created_at),
                    updated_at: datetime_to_proto_timestamp(stream.updated_at),
                })
                .collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count as i64,
                page: page as i32,
                page_size: page_size as i32,
                has_next,
            }),
        })
    }

    pub async fn append_events(
        &self,
        req: ProtoAppendEventsRequest,
    ) -> Result<ProtoAppendEventsResponse, GrpcError> {
        // JSONデシリアライズ失敗をサイレントに無視せず、INVALID_ARGUMENTとして伝播する
        let events: Vec<EventData> = req
            .events
            .into_iter()
            .map(|e| {
                let payload = serde_json::from_slice(&e.payload).map_err(|err| {
                    GrpcError::InvalidArgument(format!(
                        "イベントペイロードが不正なJSONです: {}",
                        err
                    ))
                })?;
                let metadata = e.metadata.unwrap_or_default();
                Ok(EventData {
                    event_type: e.event_type,
                    payload,
                    metadata: EventMetadata::new(
                        metadata.actor_id,
                        metadata.correlation_id,
                        metadata.causation_id,
                    ),
                })
            })
            .collect::<Result<Vec<EventData>, GrpcError>>()?;

        let input = AppendEventsInput {
            stream_id: req.stream_id,
            aggregate_type: if req.aggregate_type.trim().is_empty() {
                None
            } else {
                Some(req.aggregate_type)
            },
            events,
            expected_version: req.expected_version,
        };

        match self.append_events_uc.execute(&input).await {
            Ok(output) => {
                // REST と同様にバックグラウンドで Kafka にイベントを発行する
                spawn_publish_events_with_retry(
                    self.event_publisher.clone(),
                    output.stream_id.clone(),
                    output.events.clone(),
                );

                // シリアライズ失敗をエラーとして収集し、失敗時はINTERNALを返す
                let events = output
                    .events
                    .into_iter()
                    .map(stored_event_to_proto)
                    .collect::<Result<Vec<ProtoStoredEvent>, GrpcError>>()?;
                Ok(ProtoAppendEventsResponse {
                    stream_id: output.stream_id,
                    events,
                    current_version: output.current_version,
                })
            }
            Err(AppendEventsError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(AppendEventsError::StreamAlreadyExists(id)) => Err(GrpcError::AlreadyExists(
                format!("stream already exists: {}", id),
            )),
            Err(AppendEventsError::VersionConflict {
                stream_id,
                expected,
                actual,
            }) => Err(GrpcError::Aborted(format!(
                "version conflict for stream {}: expected {}, actual {}",
                stream_id, expected, actual
            ))),
            Err(AppendEventsError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn read_events(
        &self,
        req: ProtoReadEventsRequest,
    ) -> Result<ProtoReadEventsResponse, GrpcError> {
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = req.pagination.unwrap_or_default();
        let page = if pagination.page <= 0 {
            1
        } else {
            pagination.page as u32
        };
        let page_size = if pagination.page_size <= 0 {
            20
        } else {
            pagination.page_size as u32
        };
        let input = ReadEventsInput {
            stream_id: req.stream_id,
            from_version: req.from_version,
            to_version: req.to_version,
            event_type: req.event_type,
            page,
            page_size,
        };

        match self.read_events_uc.execute(&input).await {
            Ok(output) => {
                // シリアライズ失敗をエラーとして収集し、失敗時はINTERNALを返す
                let events = output
                    .events
                    .into_iter()
                    .map(stored_event_to_proto)
                    .collect::<Result<Vec<ProtoStoredEvent>, GrpcError>>()?;
                Ok(ProtoReadEventsResponse {
                    stream_id: output.stream_id,
                    events,
                    current_version: output.current_version,
                    pagination: Some(ProtoPaginationResult {
                        total_count: output.pagination.total_count as i64,
                        page: page as i32,
                        page_size: page_size as i32,
                        has_next: output.pagination.has_next,
                    }),
                })
            }
            Err(ReadEventsError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn read_event_by_sequence(
        &self,
        req: ProtoReadEventBySequenceRequest,
    ) -> Result<ProtoReadEventBySequenceResponse, GrpcError> {
        let input = ReadEventBySequenceInput {
            stream_id: req.stream_id,
            sequence: req.sequence,
        };

        match self.read_event_by_sequence_uc.execute(&input).await {
            Ok(event) => {
                // シリアライズ失敗時はINTERNALエラーとして伝播する
                Ok(ProtoReadEventBySequenceResponse {
                    event: Some(stored_event_to_proto(event)?),
                })
            }
            Err(ReadEventBySequenceError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(ReadEventBySequenceError::EventNotFound {
                stream_id,
                sequence,
            }) => Err(GrpcError::NotFound(format!(
                "event not found: stream={}, sequence={}",
                stream_id, sequence
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn create_snapshot(
        &self,
        req: ProtoCreateSnapshotRequest,
    ) -> Result<ProtoCreateSnapshotResponse, GrpcError> {
        // スナップショット状態のJSONデシリアライズ失敗をINVALID_ARGUMENTとして伝播する
        let state = serde_json::from_slice(&req.state).map_err(|err| {
            GrpcError::InvalidArgument(format!(
                "スナップショット状態が不正なJSONです: {}",
                err
            ))
        })?;

        let input = CreateSnapshotInput {
            stream_id: req.stream_id,
            snapshot_version: req.snapshot_version,
            aggregate_type: req.aggregate_type,
            state,
        };

        match self.create_snapshot_uc.execute(&input).await {
            Ok(output) => Ok(ProtoCreateSnapshotResponse {
                id: output.id,
                stream_id: output.stream_id,
                snapshot_version: output.snapshot_version,
                created_at: datetime_to_proto_timestamp(output.created_at),
                aggregate_type: output.aggregate_type,
            }),
            Err(CreateSnapshotError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(CreateSnapshotError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_latest_snapshot(
        &self,
        req: ProtoGetLatestSnapshotRequest,
    ) -> Result<ProtoGetLatestSnapshotResponse, GrpcError> {
        let input = GetLatestSnapshotInput {
            stream_id: req.stream_id,
        };

        match self.get_latest_snapshot_uc.execute(&input).await {
            Ok(snapshot) => {
                // スナップショット状態のシリアライズ失敗はINTERNALエラーとして伝播する
                let state = serde_json::to_vec(&snapshot.state).map_err(|err| {
                    GrpcError::Internal(format!(
                        "スナップショット状態のシリアライズに失敗しました: {}",
                        err
                    ))
                })?;
                Ok(ProtoGetLatestSnapshotResponse {
                    snapshot: Some(ProtoSnapshot {
                        id: snapshot.id,
                        stream_id: snapshot.stream_id,
                        snapshot_version: snapshot.snapshot_version,
                        aggregate_type: snapshot.aggregate_type,
                        state,
                        created_at: datetime_to_proto_timestamp(snapshot.created_at),
                    }),
                })
            }
            Err(GetLatestSnapshotError::StreamNotFound(id)) => {
                Err(GrpcError::NotFound(format!("stream not found: {}", id)))
            }
            Err(GetLatestSnapshotError::SnapshotNotFound(id)) => Err(GrpcError::NotFound(format!(
                "snapshot not found for stream: {}",
                id
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn delete_stream(
        &self,
        req: ProtoDeleteStreamRequest,
    ) -> Result<ProtoDeleteStreamResponse, GrpcError> {
        let out = self
            .delete_stream_uc
            .execute(&DeleteStreamInput {
                stream_id: req.stream_id,
            })
            .await
            .map_err(|e| match e {
                DeleteStreamError::StreamNotFound(id) => {
                    GrpcError::NotFound(format!("stream not found: {}", id))
                }
                DeleteStreamError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ProtoDeleteStreamResponse {
            success: out.success,
            message: out.message,
        })
    }
}

/// chrono::DateTime<Utc> を Proto の Timestamp メッセージに変換する。
/// 他サーバー（api-registry, event-monitor, config）と同一のパターンを使用。
fn datetime_to_proto_timestamp(dt: DateTime<Utc>) -> Option<ProtoTimestamp> {
    Some(ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

/// StoredEvent を Proto メッセージに変換する。
/// ペイロードのシリアライズ失敗はINTERNALエラーとして伝播する。
fn stored_event_to_proto(
    e: crate::domain::entity::event::StoredEvent,
) -> Result<ProtoStoredEvent, GrpcError> {
    // ペイロードのシリアライズ失敗をサイレントに無視せず、INTERNALエラーとして伝播する
    let payload = serde_json::to_vec(&e.payload).map_err(|err| {
        GrpcError::Internal(format!(
            "イベントペイロードのシリアライズに失敗しました: {}",
            err
        ))
    })?;
    Ok(ProtoStoredEvent {
        stream_id: e.stream_id,
        sequence: e.sequence,
        event_type: e.event_type,
        version: e.version,
        payload,
        metadata: Some(ProtoEventMetadata {
            actor_id: e.metadata.actor_id,
            correlation_id: e.metadata.correlation_id,
            causation_id: e.metadata.causation_id,
        }),
        occurred_at: datetime_to_proto_timestamp(e.occurred_at),
        stored_at: datetime_to_proto_timestamp(e.stored_at),
    })
}

// ---------------------------------------------------------------------------
// CRIT-004 監査対応: JSONデシリアライズエラーが適切なgRPCエラーとして伝播することを検証する。
// サイレントに無視せず、InvalidArgument として呼び出し元にエラーを返すことを確認する。
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository, MockSnapshotRepository,
    };
    use crate::infrastructure::kafka::MockEventPublisher;
    use crate::usecase::append_events::AppendEventsUseCase;
    use crate::usecase::create_snapshot::CreateSnapshotUseCase;
    use crate::usecase::delete_stream::DeleteStreamUseCase;
    use crate::usecase::get_latest_snapshot::GetLatestSnapshotUseCase;
    use crate::usecase::read_event_by_sequence::ReadEventBySequenceUseCase;
    use crate::usecase::read_events::ReadEventsUseCase;

    /// テスト用の EventStoreGrpcService を構築するヘルパー。
    /// スタブの実装をモックとして注入し、JSONパース処理だけをテストする。
    fn make_service() -> EventStoreGrpcService {
        let stream_repo = Arc::new(MockEventStreamRepository::new());
        let event_repo = Arc::new(MockEventRepository::new());
        let snapshot_repo = Arc::new(MockSnapshotRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let append_uc = Arc::new(AppendEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let read_events_uc = Arc::new(ReadEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let read_by_seq_uc = Arc::new(ReadEventBySequenceUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let create_snap_uc = Arc::new(CreateSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        ));
        let get_snap_uc = Arc::new(GetLatestSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        ));
        let delete_uc = Arc::new(DeleteStreamUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
            snapshot_repo.clone(),
        ));

        EventStoreGrpcService::new(
            append_uc,
            read_events_uc,
            read_by_seq_uc,
            create_snap_uc,
            get_snap_uc,
            delete_uc,
            stream_repo,
            publisher,
        )
    }

    /// CRIT-004 監査対応: 不正なJSONペイロードを持つイベントの append_events リクエストは
    /// InvalidArgument エラーとして伝播し、サイレントに無視されないことを確認する。
    #[tokio::test]
    async fn test_append_events_invalid_json_payload_returns_invalid_argument() {
        let svc = make_service();

        let req = ProtoAppendEventsRequest {
            stream_id: "stream-001".to_string(),
            expected_version: -1,
            aggregate_type: "TestAggregate".to_string(),
            events: vec![
                crate::proto::k1s0::system::eventstore::v1::EventData {
                    event_type: "TestEvent".to_string(),
                    // 不正なバイト列: 有効なJSONではない
                    payload: b"not-valid-json{{{".to_vec(),
                    metadata: None,
                },
            ],
        };

        let result = svc.append_events(req).await;
        assert!(result.is_err(), "不正なJSONペイロードはエラーを返すべき");
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => {
                assert!(
                    msg.contains("不正なJSON") || msg.contains("JSON"),
                    "エラーメッセージにJSON不正の旨が含まれるべき: {msg}"
                );
            }
            e => panic!("InvalidArgument を期待したが {e:?} が返った"),
        }
    }

    /// CRIT-004 監査対応: 不正なJSONバイト列を持つ create_snapshot リクエストは
    /// InvalidArgument エラーとして伝播し、サイレントに無視されないことを確認する。
    #[tokio::test]
    async fn test_create_snapshot_invalid_json_state_returns_invalid_argument() {
        let svc = make_service();

        let req = ProtoCreateSnapshotRequest {
            stream_id: "stream-001".to_string(),
            snapshot_version: 1,
            aggregate_type: "TestAggregate".to_string(),
            // 不正なバイト列: 有効なJSONではない
            state: b"not-valid-json{{{".to_vec(),
        };

        let result = svc.create_snapshot(req).await;
        assert!(result.is_err(), "不正なJSONスナップショット状態はエラーを返すべき");
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => {
                assert!(
                    msg.contains("不正なJSON") || msg.contains("JSON"),
                    "エラーメッセージにJSON不正の旨が含まれるべき: {msg}"
                );
            }
            e => panic!("InvalidArgument を期待したが {e:?} が返った"),
        }
    }

    /// CRIT-004 監査対応: 有効なJSONペイロード（空オブジェクト）は正常にパースされ、
    /// InvalidArgument エラーにならないことを確認する（誤発火防止）。
    /// ユースケース層でエラーが発生しても、JSONパースエラーではないことを検証する。
    #[tokio::test]
    async fn test_append_events_valid_json_payload_does_not_return_invalid_argument() {
        // このテストではストリームが見つからないエラー（StreamNotFound）が返ることを想定するため、
        // MockEventStreamRepository に find_by_id の期待値を設定する必要がある。
        // make_service() とは別に個別にモックを組み立てる。
        let mut stream_repo = MockEventStreamRepository::new();
        stream_repo
            .expect_find_by_id()
            .returning(|_| Ok(None)); // ストリームが存在しない → StreamNotFound エラーになる
        let stream_repo = Arc::new(stream_repo);

        let event_repo = Arc::new(MockEventRepository::new());
        let snapshot_repo = Arc::new(MockSnapshotRepository::new());
        let publisher = Arc::new(MockEventPublisher::new());

        let append_uc = Arc::new(AppendEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let read_events_uc = Arc::new(ReadEventsUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let read_by_seq_uc = Arc::new(ReadEventBySequenceUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
        ));
        let create_snap_uc = Arc::new(CreateSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        ));
        let get_snap_uc = Arc::new(GetLatestSnapshotUseCase::new(
            stream_repo.clone(),
            snapshot_repo.clone(),
        ));
        let delete_uc = Arc::new(DeleteStreamUseCase::new(
            stream_repo.clone(),
            event_repo.clone(),
            snapshot_repo.clone(),
        ));
        let svc = EventStoreGrpcService::new(
            append_uc,
            read_events_uc,
            read_by_seq_uc,
            create_snap_uc,
            get_snap_uc,
            delete_uc,
            stream_repo,
            publisher,
        );

        let req = ProtoAppendEventsRequest {
            stream_id: "stream-001".to_string(),
            expected_version: 0, // 0 を指定するとストリームが存在しないため StreamNotFound になる
            aggregate_type: "TestAggregate".to_string(),
            events: vec![
                crate::proto::k1s0::system::eventstore::v1::EventData {
                    event_type: "TestEvent".to_string(),
                    // 有効なJSON
                    payload: b"{}".to_vec(),
                    metadata: None,
                },
            ],
        };

        let result = svc.append_events(req).await;
        // ストリームが存在しないため StreamNotFound エラーが返るが、
        // InvalidArgument（JSONパースエラー）は返ってはいけない
        assert!(result.is_err(), "ストリームが存在しないためエラーが返るべき");
        assert!(
            matches!(result.unwrap_err(), GrpcError::NotFound(_)),
            "有効なJSONの場合は NotFound エラーが返り、InvalidArgument は返ってはいけない"
        );
    }
}
