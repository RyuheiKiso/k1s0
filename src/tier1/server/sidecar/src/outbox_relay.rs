// outbox_relay.rs — spec 10 テナント分離 + tier1 sidecar: Outbox → Kafka relay
// tier2 atomic_triple_write.rs の第 2 TABLE（outbox_message）を PostgreSQL から読み出し、
// Kafka idempotent producer に relay する。
// P4 invariant: rollback 時は outbox_message も rollback 済みなので配送しない。
// Debezium-compatible CloudEvents 1.0 format で送信する。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};

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
    // kafka_brokers: Kafka broker アドレス（カンマ区切り）
    pub kafka_brokers: String,
    // poll_interval_ms: outbox_message テーブルのポーリング間隔（ms）
    pub poll_interval_ms: u64,
    // max_retry_count: dead-letter に移動する前の最大 retry 回数
    pub max_retry_count: i32,
}

// OutboxRelay は PostgreSQL Outbox → Kafka relay を実装する。
// 実装方針:
//   1. PostgreSQL LISTEN/NOTIFY + SELECT FOR UPDATE SKIP LOCKED でメッセージを取得する
//   2. Kafka idempotent producer で CloudEvents 1.0 を送信する
//   3. 送信成功後に outbox_message から DELETE する
//   4. retry_count > max_retry_count で outbox_dead_letter に移動する
pub struct OutboxRelay {
    config: OutboxRelayConfig,
}

impl OutboxRelay {
    // new は OutboxRelay を構築する。
    pub fn new(config: OutboxRelayConfig) -> Self {
        Self { config }
    }

    // run は Outbox relay ループを起動する。
    // PostgreSQL LISTEN で NOTIFY を受け取るたびに SELECT FOR UPDATE SKIP LOCKED を実行する。
    pub async fn run(&self) -> Result<()> {
        info!(
            database = %self.config.database_url,
            kafka = %self.config.kafka_brokers,
            poll_ms = self.config.poll_interval_ms,
            "OutboxRelay starting"
        );
        // TODO: sqlx::PgPool を初期化して PostgreSQL に接続する
        // let pool = sqlx::PgPool::connect(&self.config.database_url).await?;
        // TODO: rdkafka::ClientConfig でインターリーブ対応の Kafka producer を初期化する
        // let producer: FutureProducer = ClientConfig::new()
        //     .set("bootstrap.servers", &self.config.kafka_brokers)
        //     .set("enable.idempotence", "true")
        //     .create()?;
        // TODO: PostgreSQL LISTEN で outbox_message テーブルの NOTIFY を受け取る
        // tokio::time::interval(Duration::from_millis(self.config.poll_interval_ms))
        info!("OutboxRelay: database and Kafka connections pending (sqlx + rdkafka integration)");
        // メイン relay ループ（TODO: LISTEN/NOTIFY + SELECT FOR UPDATE SKIP LOCKED 実装）
        loop {
            // 現時点ではポーリング間隔で待機する
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.config.poll_interval_ms
            )).await;
            debug!("OutboxRelay: poll tick (pending PostgreSQL + Kafka integration)");
        }
    }

    // relay_message は 1 メッセージを Kafka に送信して relay 結果を返す。
    pub async fn relay_message(&self, msg: &OutboxMessage) -> RelayResult {
        // TODO: rdkafka FutureProducer で送信する
        // let record = FutureRecord::to(&msg.topic)
        //     .key(&msg.partition_key)
        //     .payload(&msg.cloudevent_json);
        // match producer.send(record, Duration::from_secs(5)).await { ... }
        info!(
            idempotency_key = %msg.idempotency_key,
            tenant_id = %msg.tenant_id,
            topic = %msg.topic,
            "OutboxRelay: relaying message (Kafka integration pending)"
        );
        // 現時点では常に成功として返す（Kafka 統合後に実 send に置き換える）
        RelayResult {
            idempotency_key: msg.idempotency_key.clone(),
            success: true,
            error_message: None,
        }
    }

    // move_to_dead_letter は retry 上限を超えたメッセージを dead-letter に移動する。
    pub async fn move_to_dead_letter(&self, msg: &OutboxMessage, reason: &str) {
        // TODO: sqlx で outbox_dead_letter table に INSERT して outbox_message から DELETE する
        error!(
            idempotency_key = %msg.idempotency_key,
            retry_count = msg.retry_count,
            reason,
            "OutboxRelay: message moved to dead-letter (sqlx integration pending)"
        );
    }
}
