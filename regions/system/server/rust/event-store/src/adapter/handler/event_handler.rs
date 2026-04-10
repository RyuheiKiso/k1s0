use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use super::error::EventStoreError;
use super::AppState;
use crate::domain::entity::event::{EventData, EventMetadata};
use crate::usecase::append_events::{AppendEventsError, AppendEventsInput};
use crate::usecase::create_snapshot::{CreateSnapshotError, CreateSnapshotInput};
use crate::usecase::delete_stream::{DeleteStreamError, DeleteStreamInput};
use crate::usecase::get_latest_snapshot::{GetLatestSnapshotError, GetLatestSnapshotInput};
use crate::usecase::list_events::{ListEventsError, ListEventsInput};
use crate::usecase::list_streams::{ListStreamsError, ListStreamsInput};
use crate::usecase::read_event_by_sequence::{ReadEventBySequenceError, ReadEventBySequenceInput};
use crate::usecase::read_events::{ReadEventsError, ReadEventsInput};

/// JWT クレームからテナント ID を抽出するヘルパー。
/// クレームが存在しない場合（開発環境など）はフォールバック値 "system" を返す。
// Option<&T> の方が &Option<T> よりも慣用的（Clippy: ref_option）
fn extract_tenant_id(claims: Option<&Extension<k1s0_auth::Claims>>) -> String {
    claims
        .map_or_else(|| "system".to_string(), |ext| ext.0.tenant_id().to_string())
}

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AppendEventsRequest {
    pub stream_id: String,
    pub aggregate_type: Option<String>,
    pub expected_version: i64,
    pub events: Vec<EventDataRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EventDataRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub metadata: MetadataRequest,
}

// イベントメタデータフィールドの _id サフィックスはドメイン上の意図的な命名規則
#[allow(clippy::struct_field_names)]
#[derive(Debug, Default, Deserialize, utoipa::ToSchema)]
pub struct MetadataRequest {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AppendEventsResponse {
    pub stream_id: String,
    pub current_version: i64,
    pub events: Vec<StoredEventResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StoredEventResponse {
    pub stream_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub version: i64,
    pub payload: serde_json::Value,
    pub metadata: MetadataResponse,
    pub occurred_at: String,
    pub stored_at: String,
}

// イベントメタデータレスポンスフィールドの _id サフィックスはドメイン上の意図的な命名規則
#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MetadataResponse {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadEventsQuery {
    #[serde(default = "default_from_version")]
    pub from_version: i64,
    pub to_version: Option<i64>,
    pub event_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_from_version() -> i64 {
    1
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

// Kafka パブリッシュのリトライ初回遅延（ミリ秒）
const INITIAL_PUBLISH_RETRY_DELAY_MS: u64 = 500;
// Kafka パブリッシュのリトライ最大遅延（ミリ秒）
const MAX_PUBLISH_RETRY_DELAY_MS: u64 = 30_000;
// Kafka パブリッシュのリトライ最大試行回数。無制限リトライを防止する。
const MAX_PUBLISH_RETRY_ATTEMPTS: u32 = 10;

/// イベントの Kafka パブリッシュをバックグラウンドで実行し、失敗時はリトライする。
/// `リトライ上限（MAX_PUBLISH_RETRY_ATTEMPTS）に達した場合はエラーログを出力して終了する`。
/// LOW-010 監査対応: pool が Some の場合、リトライ上限到達時に DB へ失敗状態を記録し
/// イベント消失を防止する。成功時は 'published' に更新する。
/// tenant_id は RLS ポリシー満足のために必要（set_config でセッション変数を設定する）。
pub(crate) fn spawn_publish_events_with_retry(
    publisher: Arc<dyn crate::infrastructure::kafka::EventPublisher>,
    stream_id: String,
    tenant_id: String,
    events: Vec<crate::domain::entity::event::StoredEvent>,
    pool: Option<sqlx::PgPool>,
) {
    tokio::spawn(async move {
        let mut attempt: u32 = 0;
        let mut retry_delay = Duration::from_millis(INITIAL_PUBLISH_RETRY_DELAY_MS);
        let max_retry_delay = Duration::from_millis(MAX_PUBLISH_RETRY_DELAY_MS);

        loop {
            attempt += 1;
            match publisher.publish_events(&stream_id, &events).await {
                Ok(()) => {
                    if attempt > 1 {
                        tracing::info!(
                            stream_id = %stream_id,
                            attempts = attempt,
                            "event publish succeeded after retry"
                        );
                    }
                    // LOW-010 監査対応: パブリッシュ成功時に publish_status を 'published' に更新する。
                    if let Some(ref p) = pool {
                        if let Err(db_err) = mark_events_as_published(p, &stream_id, &tenant_id).await {
                            tracing::warn!(error = %db_err, stream_id = %stream_id, "failed to update publish_status to published");
                        }
                    }
                    break;
                }
                Err(e) => {
                    // リトライ上限に達した場合はエラーログを出力して終了する
                    if attempt >= MAX_PUBLISH_RETRY_ATTEMPTS {
                        tracing::error!(
                            error = %e,
                            stream_id = %stream_id,
                            attempts = attempt,
                            max_attempts = MAX_PUBLISH_RETRY_ATTEMPTS,
                            "failed to publish events to kafka after max retry attempts, giving up"
                        );
                        // LOW-010 監査対応: Kafka パブリッシュ失敗をDBに記録し、イベント消失を防止する。
                        // spawn_publish_failed_monitor が定期的に publish_failed を検知し、
                        // ADR-0118 の再送ジョブ（Phase B）で回収する設計。
                        if let Some(ref p) = pool {
                            if let Err(db_err) = mark_events_as_publish_failed(p, &stream_id, &tenant_id).await {
                                tracing::error!(
                                    error = %db_err,
                                    stream_id = %stream_id,
                                    "failed to mark events as publish_failed in DB"
                                );
                            }
                        }
                        break;
                    }

                    tracing::warn!(
                        error = %e,
                        stream_id = %stream_id,
                        attempts = attempt,
                        max_attempts = MAX_PUBLISH_RETRY_ATTEMPTS,
                        // LOW-008: 安全な型変換（オーバーフロー防止）
                        next_retry_ms = u64::try_from(retry_delay.as_millis()).unwrap_or(u64::MAX),
                        "failed to publish events to kafka, will retry"
                    );

                    tokio::time::sleep(retry_delay).await;
                    // 指数バックオフ（最大遅延で上限）
                    retry_delay = std::cmp::min(retry_delay.saturating_mul(2), max_retry_delay);
                }
            }
        }
    });
}

/// LOW-010 監査対応: Kafka パブリッシュ失敗時に eventstore.events テーブルの
/// publish_status を 'publish_failed' に更新してイベント消失を防止する。
/// eventstore.events には FORCE ROW LEVEL SECURITY が適用されているため、
/// トランザクション内で set_config('app.current_tenant_id', tenant_id, true) を実行して
/// RLS ポリシーを満たす必要がある（lessons.md: SET LOCAL にバインドは使えない→ set_config 使用）。
async fn mark_events_as_publish_failed(
    pool: &sqlx::PgPool,
    stream_id: &str,
    tenant_id: &str,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    // RLS ポリシー（tenant_id = current_setting('app.current_tenant_id', true)）を満たす
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE eventstore.events SET publish_status = 'publish_failed' \
         WHERE stream_id = $1 AND tenant_id = $2 AND publish_status = 'pending'"
    )
    .bind(stream_id)
    .bind(tenant_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// LOW-010 監査対応: Kafka パブリッシュ成功時に eventstore.events テーブルの
/// publish_status を 'published' に更新する。
/// eventstore.events の RLS ポリシーを満たすため set_config でテナント ID を設定する。
/// 通常フロー（pending → published）と再送ジョブ（publish_failed → published）の
/// 両方に対応するため、publish_status != 'published' の全行を対象とする。
async fn mark_events_as_published(
    pool: &sqlx::PgPool,
    stream_id: &str,
    tenant_id: &str,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    // RLS ポリシー満足のためテナント ID を設定する
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    // publish_status IN ('pending', 'publish_failed') を対象とする。
    // 通常フロー（pending）と再送ジョブ（publish_failed）の両方で呼ばれるため。
    sqlx::query(
        "UPDATE eventstore.events SET publish_status = 'published' \
         WHERE stream_id = $1 AND tenant_id = $2 \
         AND publish_status IN ('pending', 'publish_failed')"
    )
    .bind(stream_id)
    .bind(tenant_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// LOW-010 監査対応（ADR-0118 Phase A + B 統合実装）:
/// publish_failed イベントの定期監視と自動再送を行うバックグラウンドジョブ。
///
/// 設計方針（ADR-0118）:
/// - FORCE ROW LEVEL SECURITY 下では通常の COUNT/SELECT は全行が RLS で隠される。
///   migration 010 で追加した SECURITY DEFINER 関数
///   `eventstore.count_publish_failed_all_tenants()` /
///   `eventstore.list_publish_failed_events()` を経由することで、
///   関数オーナー（スーパーユーザー）権限で全テナント横断のアクセスが可能になる。
/// - 60 秒間隔でバッチ（最大 100 件）を取得し、stream_id+tenant_id 単位で
///   Kafka パブリッシュを再試行する。成功時は 'published' に更新する。
///   失敗時はログを出力し次のインターバルで再試行する（無限ループ防止）。
pub fn spawn_publish_failed_retry_job(
    pool: sqlx::PgPool,
    publisher: Arc<dyn crate::infrastructure::kafka::EventPublisher>,
) {
    /// 再送ジョブの実行間隔（秒）
    const RETRY_INTERVAL_SECS: u64 = 60;
    /// 1 バッチで取得する最大イベント件数（負荷制御）
    const BATCH_LIMIT: i32 = 100;

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(RETRY_INTERVAL_SECS));
        // 起動直後の即時実行を避けるため最初の tick をスキップする
        interval.tick().await;

        loop {
            interval.tick().await;

            // Phase A: SECURITY DEFINER 関数で全テナント横断の publish_failed 件数を取得する
            let count = match sqlx::query_scalar::<_, i64>(
                "SELECT eventstore.count_publish_failed_all_tenants()"
            )
            .fetch_one(&pool)
            .await
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to count publish_failed events"
                    );
                    continue;
                }
            };

            if count == 0 {
                // 失敗件数ゼロ: 正常、次のインターバルまでスキップ
                continue;
            }

            tracing::warn!(
                count = count,
                "publish_failed events detected: attempting retry (ADR-0118 Phase B)"
            );

            // Phase B: SECURITY DEFINER 関数でイベントを取得してリパブリッシュする
            let rows = match sqlx::query_as::<_, PublishFailedEventRow>(
                "SELECT * FROM eventstore.list_publish_failed_events($1)"
            )
            .bind(BATCH_LIMIT)
            .fetch_all(&pool)
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to fetch publish_failed events for retry"
                    );
                    continue;
                }
            };

            // stream_id + tenant_id 単位でグループ化してパブリッシュする
            let mut groups: std::collections::HashMap<
                (String, String),
                Vec<crate::domain::entity::event::StoredEvent>,
            > = std::collections::HashMap::new();

            for row in rows {
                // metadata は JSONB → EventMetadata に変換（失敗時は空メタデータ）
                let metadata = serde_json::from_value(row.r_metadata)
                    .unwrap_or_else(|_| crate::domain::entity::event::EventMetadata::new(
                        None, None, None,
                    ));
                let event = crate::domain::entity::event::StoredEvent {
                    stream_id: row.r_stream_id.clone(),
                    tenant_id: row.r_tenant_id.clone(),
                    // DB の sequence は i64 だが、ドメインでは u64 を使用する
                    sequence: u64::try_from(row.r_sequence).unwrap_or(0),
                    event_type: row.r_event_type,
                    version: row.r_version,
                    payload: row.r_payload,
                    metadata,
                    occurred_at: row.r_occurred_at,
                    stored_at: row.r_stored_at,
                };
                groups
                    .entry((row.r_stream_id, row.r_tenant_id))
                    .or_default()
                    .push(event);
            }

            for ((stream_id, tenant_id), events) in groups {
                match publisher.publish_events(&stream_id, &events).await {
                    Ok(()) => {
                        tracing::info!(
                            stream_id = %stream_id,
                            tenant_id = %tenant_id,
                            count = events.len(),
                            "retry publish succeeded for publish_failed events"
                        );
                        // 成功: publish_failed → published に更新する
                        // mark_events_as_published は IN ('pending', 'publish_failed') を対象とする
                        if let Err(e) =
                            mark_events_as_published(&pool, &stream_id, &tenant_id).await
                        {
                            tracing::warn!(
                                error = %e,
                                stream_id = %stream_id,
                                "failed to mark retried events as published"
                            );
                        }
                    }
                    Err(e) => {
                        // 失敗: ログのみ、次のインターバルで再試行する
                        tracing::warn!(
                            error = %e,
                            stream_id = %stream_id,
                            tenant_id = %tenant_id,
                            "retry publish failed, will try again at next interval"
                        );
                    }
                }
            }
        }
    });
}

/// publish_failed 再送ジョブが DB から読み取るイベント行の構造体。
/// migration 010 の SECURITY DEFINER 関数 `list_publish_failed_events` の返却型と対応する。
#[derive(sqlx::FromRow)]
struct PublishFailedEventRow {
    r_stream_id:   String,
    r_tenant_id:   String,
    r_sequence:    i64,
    r_event_type:  String,
    r_version:     i64,
    r_payload:     serde_json::Value,
    r_metadata:    serde_json::Value,
    r_occurred_at: chrono::DateTime<chrono::Utc>,
    r_stored_at:   chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadEventsResponse {
    pub stream_id: String,
    pub events: Vec<StoredEventResponse>,
    pub current_version: i64,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationResponse {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub event_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListEventsResponse {
    pub events: Vec<StoredEventResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Deserialize)]
pub struct ListStreamsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StreamResponse {
    pub id: String,
    pub aggregate_type: String,
    pub current_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListStreamsResponse {
    pub streams: Vec<StreamResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSnapshotRequest {
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotResponse {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
    pub created_at: String,
}

// --- Helpers ---

fn to_stored_event_response(
    event: &crate::domain::entity::event::StoredEvent,
) -> StoredEventResponse {
    StoredEventResponse {
        stream_id: event.stream_id.clone(),
        sequence: event.sequence,
        event_type: event.event_type.clone(),
        version: event.version,
        payload: event.payload.clone(),
        metadata: MetadataResponse {
            actor_id: event.metadata.actor_id.clone(),
            correlation_id: event.metadata.correlation_id.clone(),
            causation_id: event.metadata.causation_id.clone(),
        },
        occurred_at: event.occurred_at.to_rfc3339(),
        stored_at: event.stored_at.to_rfc3339(),
    }
}

fn to_stream_response(stream: &crate::domain::entity::event::EventStream) -> StreamResponse {
    StreamResponse {
        id: stream.id.clone(),
        aggregate_type: stream.aggregate_type.clone(),
        current_version: stream.current_version,
        created_at: stream.created_at.to_rfc3339(),
        updated_at: stream.updated_at.to_rfc3339(),
    }
}

// --- Handlers ---

/// POST /api/v1/events - Append events to a stream
#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = AppendEventsRequest,
    responses(
        (status = 201, description = "Events appended", body = AppendEventsResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Stream not found"),
        (status = 409, description = "Version conflict"),
    ),
)]
pub async fn append_events(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Json(req): Json<AppendEventsRequest>,
) -> Result<(axum::http::StatusCode, Json<AppendEventsResponse>), EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let events: Vec<EventData> = req
        .events
        .into_iter()
        .map(|e| EventData {
            event_type: e.event_type,
            payload: e.payload,
            metadata: EventMetadata::new(
                e.metadata.actor_id,
                e.metadata.correlation_id,
                e.metadata.causation_id,
            ),
        })
        .collect();

    let input = AppendEventsInput {
        stream_id: req.stream_id,
        tenant_id,
        aggregate_type: req.aggregate_type,
        events,
        expected_version: req.expected_version,
    };

    let output = state
        .append_events_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            AppendEventsError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            AppendEventsError::StreamAlreadyExists(id) => {
                EventStoreError::StreamAlreadyExists(format!("stream already exists: {id}"))
            }
            AppendEventsError::VersionConflict {
                stream_id,
                expected,
                actual,
            } => EventStoreError::VersionConflict(format!(
                "version conflict for stream {stream_id}: expected {expected}, actual {actual}"
            )),
            AppendEventsError::Validation(msg) => EventStoreError::Validation(msg),
            AppendEventsError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    // Publish in background with retry for at-least-once delivery semantics.
    // LOW-010 監査対応: db_pool を渡すことで、リトライ上限到達時に DB へ失敗状態を記録する。
    // tenant_id は mark 関数内の RLS ポリシー（set_config）に必要。
    let publisher = state.event_publisher.clone();
    let stream_id = output.stream_id.clone();
    let tenant_id = output.events.first().map(|e| e.tenant_id.clone()).unwrap_or_default();
    let events = output.events.clone();
    let pool = state.db_pool.clone();
    spawn_publish_events_with_retry(publisher, stream_id, tenant_id, events, pool);

    let event_responses: Vec<StoredEventResponse> =
        output.events.iter().map(to_stored_event_response).collect();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(AppendEventsResponse {
            stream_id: output.stream_id,
            current_version: output.current_version,
            events: event_responses,
        }),
    ))
}

/// GET /`api/v1/events/:stream_id` - Read events from a stream
#[utoipa::path(
    get,
    path = "/api/v1/events/{stream_id}",
    params(
        ("stream_id" = String, Path, description = "Stream ID"),
        ("from_version" = Option<i64>, Query, description = "Start version"),
        ("to_version" = Option<i64>, Query, description = "End version"),
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Events found", body = ReadEventsResponse),
        (status = 404, description = "Stream not found"),
    ),
)]
pub async fn read_events(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(stream_id): Path<String>,
    Query(query): Query<ReadEventsQuery>,
) -> Result<Json<ReadEventsResponse>, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = ReadEventsInput {
        stream_id,
        tenant_id,
        from_version: query.from_version,
        to_version: query.to_version,
        event_type: query.event_type,
        page: query.page,
        page_size: query.page_size,
    };

    let output = state
        .read_events_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            ReadEventsError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            ReadEventsError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    let event_responses: Vec<StoredEventResponse> =
        output.events.iter().map(to_stored_event_response).collect();

    Ok(Json(ReadEventsResponse {
        stream_id: output.stream_id,
        events: event_responses,
        current_version: output.current_version,
        pagination: PaginationResponse {
            total_count: output.pagination.total_count,
            page: output.pagination.page,
            page_size: output.pagination.page_size,
            has_next: output.pagination.has_next,
        },
    }))
}

/// GET /`api/v1/streams/:stream_id/events/:sequence` - Read one event by sequence
#[utoipa::path(
    get,
    path = "/api/v1/streams/{stream_id}/events/{sequence}",
    params(
        ("stream_id" = String, Path, description = "Stream ID"),
        ("sequence" = u64, Path, description = "Event sequence"),
    ),
    responses(
        (status = 200, description = "Event found", body = StoredEventResponse),
        (status = 404, description = "Stream or event not found"),
    ),
)]
pub async fn read_event_by_sequence(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path((stream_id, sequence)): Path<(String, u64)>,
) -> Result<Json<StoredEventResponse>, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = ReadEventBySequenceInput {
        stream_id,
        tenant_id,
        sequence,
    };

    let event = state
        .read_event_by_sequence_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            ReadEventBySequenceError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            ReadEventBySequenceError::EventNotFound {
                stream_id,
                sequence,
            } => EventStoreError::EventNotFound(format!(
                "event not found: stream={stream_id}, sequence={sequence}"
            )),
            ReadEventBySequenceError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    Ok(Json(to_stored_event_response(&event)))
}

/// GET /api/v1/events - List/query events with pagination
#[utoipa::path(
    get,
    path = "/api/v1/events",
    params(
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Events list", body = ListEventsResponse),
    ),
)]
pub async fn list_events(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<ListEventsResponse>, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = ListEventsInput {
        tenant_id,
        event_type: query.event_type,
        page: query.page,
        page_size: query.page_size,
    };

    let output = state
        .list_events_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            ListEventsError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    let event_responses: Vec<StoredEventResponse> =
        output.events.iter().map(to_stored_event_response).collect();

    Ok(Json(ListEventsResponse {
        events: event_responses,
        pagination: PaginationResponse {
            total_count: output.pagination.total_count,
            page: output.pagination.page,
            page_size: output.pagination.page_size,
            has_next: output.pagination.has_next,
        },
    }))
}

/// GET /api/v1/streams - List streams
#[utoipa::path(
    get,
    path = "/api/v1/streams",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Streams list", body = ListStreamsResponse),
    ),
)]
pub async fn list_streams(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Query(query): Query<ListStreamsQuery>,
) -> Result<Json<ListStreamsResponse>, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = ListStreamsInput {
        tenant_id,
        page: query.page,
        page_size: query.page_size,
    };

    let output = state
        .list_streams_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            ListStreamsError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    let stream_responses: Vec<StreamResponse> =
        output.streams.iter().map(to_stream_response).collect();

    Ok(Json(ListStreamsResponse {
        streams: stream_responses,
        pagination: PaginationResponse {
            total_count: output.pagination.total_count,
            page: output.pagination.page,
            page_size: output.pagination.page_size,
            has_next: output.pagination.has_next,
        },
    }))
}

/// GET /`api/v1/streams/:stream_id/snapshot` - Get stream snapshot
#[utoipa::path(
    get,
    path = "/api/v1/streams/{stream_id}/snapshot",
    params(("stream_id" = String, Path, description = "Stream ID")),
    responses(
        (status = 200, description = "Snapshot found", body = SnapshotResponse),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn get_snapshot(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(stream_id): Path<String>,
) -> Result<Json<SnapshotResponse>, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = GetLatestSnapshotInput {
        stream_id,
        tenant_id,
    };

    let snapshot = state
        .get_latest_snapshot_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            GetLatestSnapshotError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            GetLatestSnapshotError::SnapshotNotFound(id) => {
                EventStoreError::SnapshotNotFound(format!("snapshot not found for stream: {id}"))
            }
            GetLatestSnapshotError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    Ok(Json(SnapshotResponse {
        id: snapshot.id,
        stream_id: snapshot.stream_id,
        snapshot_version: snapshot.snapshot_version,
        aggregate_type: snapshot.aggregate_type,
        state: snapshot.state,
        created_at: snapshot.created_at.to_rfc3339(),
    }))
}

/// POST /`api/v1/streams/:stream_id/snapshot` - Create snapshot
#[utoipa::path(
    post,
    path = "/api/v1/streams/{stream_id}/snapshot",
    params(("stream_id" = String, Path, description = "Stream ID")),
    request_body = CreateSnapshotRequest,
    responses(
        (status = 201, description = "Snapshot created", body = SnapshotResponse),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Stream not found"),
    ),
)]
pub async fn create_snapshot(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(stream_id): Path<String>,
    Json(req): Json<CreateSnapshotRequest>,
) -> Result<(axum::http::StatusCode, Json<SnapshotResponse>), EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let response_state = req.state.clone();
    let input = CreateSnapshotInput {
        stream_id,
        tenant_id,
        snapshot_version: req.snapshot_version,
        aggregate_type: req.aggregate_type,
        state: req.state,
    };

    let output = state
        .create_snapshot_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            CreateSnapshotError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            CreateSnapshotError::Validation(msg) => EventStoreError::Validation(msg),
            CreateSnapshotError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(SnapshotResponse {
            id: output.id,
            stream_id: output.stream_id,
            snapshot_version: output.snapshot_version,
            aggregate_type: output.aggregate_type,
            state: response_state,
            created_at: output.created_at.to_rfc3339(),
        }),
    ))
}

/// DELETE /`api/v1/streams/:stream_id` - Delete a stream and all its events/snapshots
#[utoipa::path(
    delete,
    path = "/api/v1/streams/{stream_id}",
    params(("stream_id" = String, Path, description = "Stream ID")),
    responses(
        (status = 200, description = "Stream deleted"),
        (status = 404, description = "Stream not found"),
    ),
)]
pub async fn delete_stream(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(stream_id): Path<String>,
) -> Result<impl axum::response::IntoResponse, EventStoreError> {
    // テナント分離のため Claims から tenant_id を取得する（ADR-0106）
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = DeleteStreamInput {
        stream_id,
        tenant_id,
    };

    state
        .delete_stream_uc
        .execute(&input)
        .await
        .map_err(|e| match e {
            DeleteStreamError::StreamNotFound(id) => {
                EventStoreError::StreamNotFound(format!("stream not found: {id}"))
            }
            DeleteStreamError::Internal(msg) => EventStoreError::Internal(msg),
        })?;

    Ok((
        axum::http::StatusCode::OK,
        Json(serde_json::json!({"success": true})),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler;
    use crate::domain::entity::event::{
        EventMetadata as DomainMeta, EventStream, Snapshot, StoredEvent,
    };
    use crate::domain::repository::event_repository::{
        MockEventRepository, MockEventStreamRepository, MockSnapshotRepository,
    };
    use crate::infrastructure::kafka::MockEventPublisher;
    use crate::usecase::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_test_state(
        stream_repo: MockEventStreamRepository,
        event_repo: MockEventRepository,
        snapshot_repo: MockSnapshotRepository,
    ) -> AppState {
        let stream: Arc<dyn crate::domain::repository::EventStreamRepository> =
            Arc::new(stream_repo);
        let event: Arc<dyn crate::domain::repository::EventRepository> = Arc::new(event_repo);
        let snap: Arc<dyn crate::domain::repository::SnapshotRepository> = Arc::new(snapshot_repo);

        let mut publisher = MockEventPublisher::new();
        publisher.expect_publish_events().returning(|_, _| Ok(()));
        publisher
            .expect_health_check()
            .times(0..)
            .returning(|| Ok(()));
        let publisher: Arc<dyn crate::infrastructure::kafka::EventPublisher> = Arc::new(publisher);

        AppState {
            append_events_uc: Arc::new(AppendEventsUseCase::new(stream.clone(), event.clone())),
            read_events_uc: Arc::new(ReadEventsUseCase::new(stream.clone(), event.clone())),
            read_event_by_sequence_uc: Arc::new(ReadEventBySequenceUseCase::new(
                stream.clone(),
                event.clone(),
            )),
            list_events_uc: Arc::new(crate::usecase::ListEventsUseCase::new(event.clone())),
            list_streams_uc: Arc::new(crate::usecase::ListStreamsUseCase::new(stream.clone())),
            create_snapshot_uc: Arc::new(CreateSnapshotUseCase::new(stream.clone(), snap.clone())),
            get_latest_snapshot_uc: Arc::new(GetLatestSnapshotUseCase::new(
                stream.clone(),
                snap.clone(),
            )),
            delete_stream_uc: Arc::new(crate::usecase::DeleteStreamUseCase::new(
                stream.clone(),
                event.clone(),
                snap.clone(),
            )),
            stream_repo: stream,
            event_publisher: publisher,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
                "k1s0-event-store-server-test",
            )),
            auth_state: None,
            // インメモリ（dev/test）モードでは DB 接続不要のため None を設定する
            db_pool: None,
        }
    }

    fn make_stream() -> EventStream {
        EventStream {
            id: "order-001".to_string(),
            tenant_id: "system".to_string(),
            aggregate_type: "Order".to_string(),
            current_version: 3,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_event(seq: u64) -> StoredEvent {
        StoredEvent::new(
            "order-001".to_string(),
            "system".to_string(),
            seq,
            "OrderPlaced".to_string(),
            // LOW-008: 安全な型変換（オーバーフロー防止）
            i64::try_from(seq).unwrap_or(i64::MAX),
            serde_json::json!({}),
            DomainMeta::new(None, None, None),
        )
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_test_state(
            MockEventStreamRepository::new(),
            MockEventRepository::new(),
            MockSnapshotRepository::new(),
        );
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("healthzリクエストの構築に失敗"),
            )
            .await
            .expect("healthzリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readyz() {
        let mut stream_repo = MockEventStreamRepository::new();
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        let state = make_test_state(
            stream_repo,
            MockEventRepository::new(),
            MockSnapshotRepository::new(),
        );
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .expect("readyzリクエストの構築に失敗"),
            )
            .await
            .expect("readyzリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_append_events_new_stream() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_, _| Ok(None));
        stream_repo.expect_create().returning(|_| Ok(()));
        stream_repo
            .expect_update_version()
            .returning(|_, _, _| Ok(()));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        event_repo.expect_append().returning(|_, sid, events| {
            Ok(events
                .into_iter()
                .enumerate()
                .map(|(i, mut e)| {
                    // LOW-008: 安全な型変換（オーバーフロー防止）
                    e.sequence = u64::try_from(i).unwrap_or(u64::MAX).saturating_add(1);
                    e.stream_id = sid.to_string();
                    e
                })
                .collect())
        });
        event_repo
            .expect_find_all()
            .returning(|_, _, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "stream_id": "order-001",
            "expected_version": -1,
            "events": [{
                "event_type": "OrderPlaced",
                "payload": {"order_id": "o-1"},
                "metadata": {"actor_id": "user-1"}
            }]
        });

        // イベント追加リクエストを送信する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&body).expect("リクエストボディのJSON変換に失敗"),
                    ))
                    .expect("append eventsリクエストの構築に失敗"),
            )
            .await
            .expect("append eventsリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_read_events_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        event_repo
            .expect_find_by_stream()
            .returning(|_, _, _, _, _, _, _| Ok((vec![make_event(1), make_event(2)], 2)));
        event_repo
            .expect_find_all()
            .returning(|_, _, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events/order-001")
                    .body(Body::empty())
                    .expect("read eventsリクエストの構築に失敗"),
            )
            .await
            .expect("read eventsリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_read_events_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_, _| Ok(None));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events/order-999")
                    .body(Body::empty())
                    .expect("read events not_foundリクエストの構築に失敗"),
            )
            .await
            .expect("read events not_foundリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_events() {
        let stream_repo = MockEventStreamRepository::new();
        let mut event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        event_repo
            .expect_find_all()
            .returning(|_, _, _, _| Ok((vec![make_event(1)], 1)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/events")
                    .body(Body::empty())
                    .expect("list eventsリクエストの構築に失敗"),
            )
            .await
            .expect("list eventsリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_streams() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![make_stream()], 1)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams")
                    .body(Body::empty())
                    .expect("list streamsリクエストの構築に失敗"),
            )
            .await
            .expect("list streamsリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        snapshot_repo.expect_find_latest().returning(|_, _| {
            Ok(Some(Snapshot::new(
                "snap_001".to_string(),
                "order-001".to_string(),
                "system".to_string(),
                3,
                "Order".to_string(),
                serde_json::json!({"status": "shipped"}),
            )))
        });

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams/order-001/snapshot")
                    .body(Body::empty())
                    .expect("get snapshotリクエストの構築に失敗"),
            )
            .await
            .expect("get snapshotリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_snapshot_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        snapshot_repo
            .expect_find_latest()
            .returning(|_, _| Ok(None));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/streams/order-001/snapshot")
                    .body(Body::empty())
                    .expect("get snapshot not_foundリクエストの構築に失敗"),
            )
            .await
            .expect("get snapshot not_foundリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_snapshot_success() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let mut snapshot_repo = MockSnapshotRepository::new();

        stream_repo
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(make_stream())));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));
        snapshot_repo.expect_create().returning(|_| Ok(()));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "snapshot_version": 2,
            "aggregate_type": "Order",
            "state": {"status": "shipped"}
        });

        // スナップショット作成リクエストを送信する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/streams/order-001/snapshot")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&body).expect("リクエストボディのJSON変換に失敗"),
                    ))
                    .expect("create snapshotリクエストの構築に失敗"),
            )
            .await
            .expect("create snapshotリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_snapshot_stream_not_found() {
        let mut stream_repo = MockEventStreamRepository::new();
        let event_repo = MockEventRepository::new();
        let snapshot_repo = MockSnapshotRepository::new();

        stream_repo.expect_find_by_id().returning(|_, _| Ok(None));
        stream_repo
            .expect_list_all()
            .returning(|_, _, _| Ok((vec![], 0)));

        let state = make_test_state(stream_repo, event_repo, snapshot_repo);
        let app = handler::router(state);

        let body = serde_json::json!({
            "snapshot_version": 2,
            "aggregate_type": "Order",
            "state": {"status": "shipped"}
        });

        // 存在しないストリームへのスナップショット作成リクエスト
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/streams/order-999/snapshot")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&body).expect("リクエストボディのJSON変換に失敗"),
                    ))
                    .expect("create snapshot not_foundリクエストの構築に失敗"),
            )
            .await
            .expect("create snapshot not_foundリクエストの送信に失敗");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
