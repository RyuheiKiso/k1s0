// messaging_bridge.rs — spec 01 Bidi §adapter messaging_bridge
// Kafka producer adapter（v1_event_feed / v1_live_snapshot / v1_bulk_upload）。
// v1_interactive と v1_alert は partition_key=session_id 制約で semantics が担保できない。
// idempotent producer + partition_key=session_id + CloudEvents 1.0 envelope を実装する。

use serde::{Deserialize, Serialize};
use super::AdapterManifest;

// MANIFEST は messaging_bridge adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "messaging_bridge",
    // v1_interactive は双方向 semantics が担保できないため not_applicable
    // v1_alert は lag≤200ms が Kafka broker round-trip で保証困難なため not_applicable
    supports: &[
        "v1_event_feed",     // Domain Event 配信（partition_key=session_id、ordering=SESSION_ORDERED per-partition）
        "v1_live_snapshot",  // 最新値表示（log-compacted topic、ordering=UNORDERED）
        "v1_bulk_upload",    // 大量データ投入（at-least-once、idempotent producer）
    ],
    // Kafka broker が存在する環境でのみ使用可能（fallback なし）
    requires_fallback: false,
    // Kafka idempotent producer + partition_key=session_id が必須
    constraints: "kafka_idempotent_producer=required, partition_key=session_id, log_compacted=for_live_snapshot",
};

// KafkaMessageEnvelope は Kafka に送信するメッセージの envelope を宣言する。
// CloudEvents 1.0 形式で Debezium CDC connector と互換性を持つ。
#[derive(Serialize, Deserialize, Debug)]
pub struct KafkaMessageEnvelope {
    // spec_version: CloudEvents 1.0 バージョン
    pub spec_version: String,
    // id: メッセージ識別子（UUID v7、idempotency_key と一致させる）
    pub id: String,
    // source: 送信元（例: k1s0://tier1/gateway/messaging_bridge）
    pub source: String,
    // event_type: イベント種別（例: "tier1.event_feed.v1"）
    pub event_type: String,
    // conformance_class: 送信している Bidi class
    pub conformance_class: String,
    // session_id: Kafka partition_key として使用する（SESSION_ORDERED 保証）
    pub session_id: String,
    // tenant_id: tier2 RLS と整合する tenant_id（AT REST での暗号化対象）
    pub tenant_id: String,
    // data: 実際のペイロード（中立形式、PII は暗号化済み）
    pub data: serde_json::Value,
    // data_content_type: ペイロードの MIME type
    pub data_content_type: String,
}

// MessagingBridgeAdapter は Kafka producer adapter。
pub struct MessagingBridgeAdapter;

impl MessagingBridgeAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }

    // topic_name は conformance_class に応じた Kafka topic 名を返す。
    // log-compacted topic は v1_live_snapshot に対してのみ適用する。
    pub fn topic_name(conformance_class: &str, tenant_id: &str) -> String {
        // tenant_id をプレフィックスにして per-tenant topic を生成する
        match conformance_class {
            // v1_event_feed は通常の retention topic
            "v1_event_feed" => format!("{tenant_id}.event_feed"),
            // v1_live_snapshot は log-compacted topic（latest-wins を Kafka で実現）
            "v1_live_snapshot" => format!("{tenant_id}.live_snapshot.compact"),
            // v1_bulk_upload は at-least-once の大容量 ingestion topic
            "v1_bulk_upload" => format!("{tenant_id}.bulk_upload"),
            // MANIFEST.supports に含まれない class は到達しない
            _ => format!("{tenant_id}.default"),
        }
    }
}
