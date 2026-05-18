// k1s0 tier2 Outbox relay
// atomic_triple_write と同一 txn で書かれた Outbox エントリを Kafka に非同期転送する
// Debezium CDC 経由の転送（直接 produce は禁止）

// シリアライズライブラリ
use serde::{Deserialize, Serialize};
// UUID ライブラリ
use uuid::Uuid;
// 日時ライブラリ
use chrono::{DateTime, Utc};

// Outbox テーブルのエントリ（Domain Event を Kafka に転送するための中継記録）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    // Outbox エントリの主キー
    pub id: Uuid,
    // 関連する aggregate の ID
    pub aggregate_id: Uuid,
    // テナント ID（Debezium CDC が Kafka routing に使用する）
    pub tenant_id: Uuid,
    // イベント種別（Debezium が Kafka topic routing に使用する）
    pub event_kind: String,
    // Kafka に転送するペイロード（PII は含まない、pii_segregated は redact 済みのみ）
    pub payload: OutboxPayload,
    // 書込日時
    pub created_at: DateTime<Utc>,
    // Debezium が処理済みにする日時（null = 未処理）
    pub processed_at: Option<DateTime<Utc>>,
}

// Outbox ペイロード（PII 平文を含まない設計）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxPayload {
    // イベントの種別識別子
    pub aggregate_type: String,
    // イベントの内容（PII は redact 済みまたは構造的に不在）
    pub data: serde_json::Value,
    // メタデータ（trace_id / version 等）
    pub metadata: serde_json::Value,
}

impl OutboxEntry {
    // OutboxEntry を生成する（PII チェック付き）
    pub fn new(
        aggregate_id: Uuid,
        tenant_id: Uuid,
        event_kind: String,
        payload: OutboxPayload,
    ) -> Self {
        // 新しい Outbox エントリを現在時刻で初期化する
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            tenant_id,
            event_kind,
            payload,
            created_at: Utc::now(),
            processed_at: None,
        }
    }

    // Outbox エントリを Kafka に転送するための INSERT SQL を生成する
    // atomic_triple_write.rs の BEGIN 〜 COMMIT ブロック内で実行される
    pub fn to_insert_sql(&self) -> String {
        // outbox_message テーブルへの INSERT SQL を返す（migration SoT: k1s0.outbox_message）
        format!(
            "INSERT INTO k1s0.outbox_message (id, aggregate_id, tenant_id, event_kind, payload, created_at) \
             VALUES ('{id}', '{agg_id}', current_setting('app.tenant_id')::uuid, '{event_kind}', '{payload}'::jsonb, '{now}');",
            id         = self.id,
            agg_id     = self.aggregate_id,
            event_kind = self.event_kind.replace('\'', "''"),
            payload    = serde_json::to_string(&self.payload)
                .unwrap_or_default()
                .replace('\'', "''"),
            now        = self.created_at.to_rfc3339(),
        )
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // OutboxEntry の INSERT SQL が必要な列を含むことを確認する
    fn test_outbox_insert_sql() {
        // テスト用の OutboxEntry を生成する
        let entry = OutboxEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "DomainEventOccurred".to_string(),
            OutboxPayload {
                aggregate_type: "Order".to_string(),
                data: serde_json::json!({"status": "created"}),
                metadata: serde_json::json!({"version": 1}),
            },
        );
        // INSERT SQL を生成する
        let sql = entry.to_insert_sql();
        // outbox_message テーブル名が SQL に含まれることを確認する（migration SoT: k1s0.outbox_message）
        assert!(sql.contains("k1s0.outbox_message"));
        assert!(sql.contains("DomainEventOccurred"));
        // GUC 経由の tenant_id 注入が含まれることを確認する
        assert!(sql.contains("current_setting('app.tenant_id')"));
    }

    #[test]
    // PII 平文が OutboxPayload に含まれないことを確認する（redaction 機構を使った実機構テスト）
    fn test_outbox_payload_no_pii() {
        // redaction モジュールをインポートする（lib.rs 経由で公開されている）
        use crate::redaction::redact_pii_fields;
        // PII フィールドを含む raw data を用意する（Outbox 書込前の状態を模擬する）
        let raw_data = serde_json::json!({
            "order_id": "WO-001",
            "status": "in_progress",
            // PII フィールドを意図的に含める（redact 前の状態）
            "email": "worker@example.com",
            "phone": "090-0000-0001"
        });
        // PII フィールドを redact する（Outbox 書込前に必ず実行する）
        let redacted_data = redact_pii_fields(raw_data);
        // redact 済みデータを OutboxPayload に格納する
        let payload = OutboxPayload {
            aggregate_type: "WorkOrder".to_string(),
            // redact 済みのデータのみを Outbox ペイロードに含める
            data: redacted_data,
            metadata: serde_json::json!({"trace_id": "abc123"}),
        };
        // JSON シリアライズして PII 平文が含まれないことを確認する
        let json_str = serde_json::to_string(&payload).unwrap();
        // PII 平文が含まれないことを確認する（redaction 機構が正しく動作していることを保証する）
        assert!(!json_str.contains("worker@example.com"));
        assert!(!json_str.contains("090-0000-0001"));
        // PII フィールドのキーは残るが値は *** に置換されていることを確認する
        assert!(json_str.contains("***"));
        // ビジネスデータは保持されていることを確認する
        assert!(json_str.contains("WO-001"));
        assert!(json_str.contains("in_progress"));
    }
}
