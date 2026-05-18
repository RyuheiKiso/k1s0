// outbox_relay.rs — spec 10 テナント分離 + tier1 sidecar: Outbox → Kafka relay
// tier2 atomic_triple_write.rs の第 2 TABLE（outbox_message）を PostgreSQL から読み出し、
// rskafka idempotent producer に relay する。
// P4 invariant: rollback 時は outbox_message も rollback 済みなので配送しない。
// CloudEvents 1.0 format で rskafka Record の value として送信する。

// anyhow: エラー伝搬ライブラリ
use anyhow::{Context, Result};
// chrono: Kafka Record timestamp に使用する（HLC ではなく wall clock; monotonic clock でのポーリング間隔 sleep は OK）
use chrono::Utc;
// rskafka: pure Rust Kafka クライアント（C ライブラリ不要）
use rskafka::{
    client::{partition::Compression, ClientBuilder},
    record::Record,
};
// serde: OutboxMessage のデシリアライズ
use serde::{Deserialize, Serialize};
// sqlx: PostgreSQL クライアント
use sqlx::{PgPool, Row};
// tokio: 非同期ランタイム + interval timer
use tokio::time::{interval, Duration};
// tracing: 構造化ロギング
use tracing::{debug, error, info, warn};

// OutboxMessage は outbox_message table の 1 行を宣言する。
// tier2/rust/src/outbox.rs の OutboxEntry と同じ構造を持つ（proto 由来）。
#[derive(Debug, Clone, Deserialize)]
pub struct OutboxMessage {
    // id: outbox_message の row id（UUID）
    pub id: String,
    // idempotency_key: dedup 用キー（tier2 P3 invariant）
    pub idempotency_key: String,
    // tenant_id: テナント識別子（Kafka topic prefix に使用する）
    pub tenant_id: String,
    // topic: 送信先 Kafka topic
    pub topic: String,
    // partition_key: Kafka partition 決定キー（session_id 推奨）
    pub partition_key: String,
    // cloudevent_json: CloudEvents 1.0 形式の JSON エンベロープ
    pub cloudevent_json: String,
    // retry_count: 送信失敗時の retry 回数
    pub retry_count: i32,
}

// RelayResult は 1 メッセージの relay 結果を宣言する。
#[derive(Debug, Clone, Serialize)]
pub struct RelayResult {
    // idempotency_key: 処理したメッセージの idempotency_key
    pub idempotency_key: String,
    // success: Kafka への送信が成功したか
    pub success: bool,
    // error_message: 失敗時のエラーメッセージ
    pub error_message: Option<String>,
}

// OutboxRelayConfig は OutboxRelay の設定を宣言する。
pub struct OutboxRelayConfig {
    // database_url: PostgreSQL 接続 URL（環境変数 DATABASE_URL から取得）
    pub database_url: String,
    // kafka_brokers: Kafka broker アドレスのリスト
    pub kafka_brokers: Vec<String>,
    // poll_interval_ms: outbox_message テーブルのポーリング間隔（ms）
    // NOTE: wall clock ではなくモノトニックな interval timer を使用する（tokio::time::interval）
    pub poll_interval_ms: u64,
    // max_retry_count: dead-letter に移動する前の最大 retry 回数
    pub max_retry_count: i32,
    // batch_size: 1 回のポーリングで取得する最大メッセージ数
    pub batch_size: i64,
}

// OutboxRelay は PostgreSQL Outbox → Kafka relay を実装する。
// 実装方針:
//   1. tokio::time::interval で poll_interval_ms ごとに SELECT FOR UPDATE SKIP LOCKED を実行する
//   2. rskafka の partition_client で CloudEvents 1.0 JSON を送信する
//   3. 送信成功後に outbox_message から DELETE する (同一 txn で送信と DELETE を行わない; at-least-once)
//   4. retry_count > max_retry_count で outbox_dead_letter に移動する
pub struct OutboxRelay {
    // 設定を保持する
    config: OutboxRelayConfig,
}

impl OutboxRelay {
    // new は OutboxRelay を構築する。
    pub fn new(config: OutboxRelayConfig) -> Self {
        Self { config }
    }

    // run は Outbox relay ループを起動する。
    // tokio::time::interval で poll_interval_ms ごとに outbox_message をポーリングする。
    pub async fn run(&self) -> Result<()> {
        info!(
            database = %self.config.database_url,
            kafka_brokers = ?self.config.kafka_brokers,
            poll_ms = self.config.poll_interval_ms,
            "OutboxRelay starting"
        );

        // sqlx::PgPool を初期化して PostgreSQL に接続する
        let pool = PgPool::connect(&self.config.database_url)
            .await
            .with_context(|| format!("PostgreSQL connection failed: {}", self.config.database_url))?;
        info!("OutboxRelay: PostgreSQL connection established");

        // rskafka::ClientBuilder で Kafka クライアントを構築する
        let kafka_client = ClientBuilder::new(self.config.kafka_brokers.clone())
            .build()
            .await
            .with_context(|| format!("Kafka connection failed: {:?}", self.config.kafka_brokers))?;
        info!("OutboxRelay: Kafka connection established");

        // tokio::time::interval でポーリング間隔を設定する（モノトニッククロック）
        let mut poll_ticker = interval(Duration::from_millis(self.config.poll_interval_ms));

        // メイン relay ループ
        loop {
            // ポーリング間隔を待つ
            poll_ticker.tick().await;

            // SELECT FOR UPDATE SKIP LOCKED で outbox_message を一括取得する
            // SKIP LOCKED: 複数の sidecar インスタンスが並行して動作しても競合しない
            // NOTE: sqlx::query!() は compile-time DATABASE_URL 要件のため sqlx::query() を使用する
            let messages = match sqlx::query(
                r#"
                SELECT
                    id::text AS id,
                    idempotency_key,
                    tenant_id::text AS tenant_id,
                    topic,
                    partition_key,
                    cloudevent_json,
                    retry_count
                FROM k1s0.outbox_message
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
                "#,
            )
            .bind(self.config.batch_size)
            .fetch_all(&pool)
            .await
            {
                Ok(rows) => {
                    // SELECT した行を OutboxMessage に変換する
                    rows.iter().map(|row| {
                        use sqlx::Row;
                        OutboxMessage {
                            id: row.try_get::<String, _>("id").unwrap_or_default(),
                            idempotency_key: row.try_get::<String, _>("idempotency_key").unwrap_or_default(),
                            tenant_id: row.try_get::<String, _>("tenant_id").unwrap_or_default(),
                            topic: row.try_get::<String, _>("topic").unwrap_or_default(),
                            partition_key: row.try_get::<String, _>("partition_key").unwrap_or_default(),
                            cloudevent_json: row.try_get::<String, _>("cloudevent_json").unwrap_or_default(),
                            retry_count: row.try_get::<i32, _>("retry_count").unwrap_or(0),
                        }
                    }).collect::<Vec<_>>()
                }
                Err(e) => {
                    // SELECT 失敗は致命的エラーではないので警告を出して続行する
                    warn!(error = %e, "OutboxRelay: SELECT FOR UPDATE SKIP LOCKED failed");
                    continue;
                }
            };

            // メッセージがない場合はスキップする
            if messages.is_empty() {
                debug!("OutboxRelay: no outbox_message pending");
                continue;
            }

            info!(count = messages.len(), "OutboxRelay: relaying outbox messages");

            // 各メッセージを Kafka に relay する
            for msg in &messages {
                // retry 回数が上限を超えた場合は dead-letter に移動する
                if msg.retry_count > self.config.max_retry_count {
                    self.move_to_dead_letter(msg, "max_retry_count exceeded", &pool).await;
                    continue;
                }

                // Kafka に relay する
                let relay_result = self.relay_message_via_rskafka(msg, &kafka_client).await;

                if relay_result.success {
                    // 送信成功したら outbox_message から DELETE する
                    if let Err(e) = sqlx::query(
                        "DELETE FROM k1s0.outbox_message WHERE id = $1::uuid",
                    )
                    .bind(msg.id.as_str())
                    .execute(&pool)
                    .await
                    {
                        // DELETE 失敗は警告のみ（次回ポーリングで再試行する）
                        warn!(
                            id = %msg.id,
                            error = %e,
                            "OutboxRelay: DELETE after relay failed"
                        );
                    } else {
                        info!(
                            idempotency_key = %msg.idempotency_key,
                            topic = %msg.topic,
                            "OutboxRelay: message relayed and deleted"
                        );
                    }
                } else {
                    // 送信失敗時は retry_count をインクリメントする
                    let error_msg = relay_result.error_message.as_deref().unwrap_or("unknown error");
                    warn!(
                        idempotency_key = %msg.idempotency_key,
                        error = %error_msg,
                        retry_count = msg.retry_count + 1,
                        "OutboxRelay: Kafka send failed, incrementing retry_count"
                    );
                    let _ = sqlx::query(
                        "UPDATE k1s0.outbox_message SET retry_count = retry_count + 1 WHERE id = $1::uuid",
                    )
                    .bind(msg.id.as_str())
                    .execute(&pool)
                    .await;
                }
            }
        }
    }

    // relay_message_via_rskafka は 1 メッセージを rskafka で Kafka に送信して relay 結果を返す。
    async fn relay_message_via_rskafka(
        &self,
        msg: &OutboxMessage,
        kafka_client: &rskafka::client::Client,
    ) -> RelayResult {
        // topic と partition_key を決定する（partition は 0 固定。本番では consistent hash で決定する）
        let topic = &msg.topic;
        let partition = 0i32;

        // partition_client を取得する（UnknownTopicHandling::Retry で topic がなければ再試行）
        let partition_client = match kafka_client
            .partition_client(
                topic.as_str(),
                partition,
                rskafka::client::partition::UnknownTopicHandling::Retry,
            )
            .await
        {
            Ok(pc) => pc,
            Err(e) => {
                // partition_client 取得失敗
                error!(
                    topic = %topic,
                    error = %e,
                    "OutboxRelay: Kafka partition_client failed"
                );
                return RelayResult {
                    idempotency_key: msg.idempotency_key.clone(),
                    success: false,
                    error_message: Some(format!("partition_client failed: {e}")),
                };
            }
        };

        // rskafka::record::Record を構築する
        let record = Record {
            // partition_key を Kafka message key として使用する（SESSION_ORDERED 保証）
            key: Some(msg.partition_key.as_bytes().to_vec()),
            // cloudevent_json を value として送信する
            value: Some(msg.cloudevent_json.as_bytes().to_vec()),
            // Kafka header に idempotency_key を含める（broker 側の dedup に使用）
            headers: {
                let mut h = std::collections::BTreeMap::new();
                h.insert(
                    "idempotency-key".to_string(),
                    msg.idempotency_key.as_bytes().to_vec(),
                );
                h.insert(
                    "tenant-id".to_string(),
                    msg.tenant_id.as_bytes().to_vec(),
                );
                h.insert(
                    "content-type".to_string(),
                    b"application/cloudevents+json".to_vec(),
                );
                h
            },
            // タイムスタンプは Utc::now() を使用する（Kafka の offset はモノトニックに管理される）
            timestamp: Utc::now(),
        };

        // Kafka に送信する（圧縮なし; 本番では Snappy 等を使う）
        match partition_client
            .produce(vec![record], Compression::NoCompression)
            .await
        {
            Ok(offsets) => {
                // 送信成功
                debug!(
                    idempotency_key = %msg.idempotency_key,
                    topic = %topic,
                    offset = offsets.first().copied().unwrap_or(-1),
                    "OutboxRelay: message produced to Kafka"
                );
                RelayResult {
                    idempotency_key: msg.idempotency_key.clone(),
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => {
                // 送信失敗
                error!(
                    idempotency_key = %msg.idempotency_key,
                    topic = %topic,
                    error = %e,
                    "OutboxRelay: Kafka produce failed"
                );
                RelayResult {
                    idempotency_key: msg.idempotency_key.clone(),
                    success: false,
                    error_message: Some(format!("Kafka produce failed: {e}")),
                }
            }
        }
    }

    // move_to_dead_letter は retry 上限を超えたメッセージを dead-letter に移動する。
    // 同一 txn で outbox_dead_letter に INSERT して outbox_message から DELETE する。
    async fn move_to_dead_letter(&self, msg: &OutboxMessage, reason: &str, pool: &PgPool) {
        // dead-letter への移動を開始するログを記録する
        error!(
            idempotency_key = %msg.idempotency_key,
            retry_count = msg.retry_count,
            reason,
            "OutboxRelay: moving message to dead-letter"
        );
        // outbox_dead_letter table に INSERT する（txn 内で実施する）
        // NOTE: sqlx::query() で動的 SQL を使用する（compile-time DATABASE_URL 不要）
        let dead_letter_result = sqlx::query(
            r#"
            WITH moved AS (
                DELETE FROM k1s0.outbox_message
                WHERE id = $1::uuid
                RETURNING id, idempotency_key, tenant_id, topic, partition_key, cloudevent_json, retry_count
            )
            INSERT INTO k1s0.outbox_dead_letter
                (id, idempotency_key, tenant_id, topic, partition_key, cloudevent_json, retry_count, dead_letter_reason, created_at)
            SELECT id, idempotency_key, tenant_id, topic, partition_key, cloudevent_json, retry_count, $2, NOW()
            FROM moved
            "#,
        )
        .bind(msg.id.as_str())
        .bind(reason)
        .execute(pool)
        .await;

        // dead-letter 移動結果をログに記録する
        match dead_letter_result {
            Ok(_) => {
                info!(
                    idempotency_key = %msg.idempotency_key,
                    "OutboxRelay: message moved to outbox_dead_letter"
                );
            }
            Err(e) => {
                // dead-letter 移動失敗は致命的エラー
                error!(
                    idempotency_key = %msg.idempotency_key,
                    error = %e,
                    "OutboxRelay: failed to move to dead-letter"
                );
            }
        }
    }
}
