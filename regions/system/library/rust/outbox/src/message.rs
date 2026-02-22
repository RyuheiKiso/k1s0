use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OutboxStatus はアウトボックスメッセージの処理ステータスを表す。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutboxStatus {
    /// 未処理（初期状態）
    Pending,
    /// 処理中
    Processing,
    /// 発行完了
    Delivered,
    /// 発行失敗（リトライ対象）
    Failed,
    /// 最大リトライ回数超過（Dead Letter）
    DeadLetter,
}

impl OutboxStatus {
    /// ステータスを文字列に変換する（DB保存用）。
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboxStatus::Pending => "PENDING",
            OutboxStatus::Processing => "PROCESSING",
            OutboxStatus::Delivered => "DELIVERED",
            OutboxStatus::Failed => "FAILED",
            OutboxStatus::DeadLetter => "DEAD_LETTER",
        }
    }

    /// 文字列からステータスを復元する（DB読み込み用）。
    pub fn from_str(s: &str) -> Self {
        match s {
            "PENDING" => OutboxStatus::Pending,
            "PROCESSING" => OutboxStatus::Processing,
            "DELIVERED" => OutboxStatus::Delivered,
            "FAILED" => OutboxStatus::Failed,
            "DEAD_LETTER" => OutboxStatus::DeadLetter,
            _ => OutboxStatus::Pending,
        }
    }
}

/// OutboxMessage はアウトボックステーブルに格納するメッセージを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxMessage {
    /// メッセージの一意識別子
    pub id: Uuid,
    /// 発行先 Kafka トピック
    pub topic: String,
    /// パーティションキー
    pub partition_key: String,
    /// メッセージペイロード（JSON）
    pub payload: serde_json::Value,
    /// 処理ステータス
    pub status: OutboxStatus,
    /// リトライ回数
    pub retry_count: u32,
    /// 最大リトライ回数
    pub max_retries: u32,
    /// 最終エラーメッセージ
    pub last_error: Option<String>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 次回処理予定日時（リトライバックオフ用）
    pub process_after: DateTime<Utc>,
}

impl OutboxMessage {
    /// 新しいアウトボックスメッセージを生成する。
    pub fn new(
        topic: impl Into<String>,
        partition_key: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            topic: topic.into(),
            partition_key: partition_key.into(),
            payload,
            status: OutboxStatus::Pending,
            retry_count: 0,
            max_retries: 3,
            last_error: None,
            created_at: now,
            process_after: now,
        }
    }

    /// メッセージを処理中状態に遷移する。
    pub fn mark_processing(&mut self) {
        self.status = OutboxStatus::Processing;
    }

    /// メッセージを配信完了状態に遷移する。
    pub fn mark_delivered(&mut self) {
        self.status = OutboxStatus::Delivered;
    }

    /// メッセージを失敗状態に遷移し、リトライ回数をインクリメントする。
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.retry_count += 1;
        self.last_error = Some(error.into());
        if self.retry_count >= self.max_retries {
            self.status = OutboxStatus::DeadLetter;
        } else {
            self.status = OutboxStatus::Failed;
            // Exponential backoff: 2^retry_count 秒後に再処理
            let delay_secs = 2u64.pow(self.retry_count);
            self.process_after = Utc::now()
                + chrono::Duration::seconds(delay_secs as i64);
        }
    }

    /// メッセージが処理可能かどうか判定する。
    pub fn is_processable(&self) -> bool {
        matches!(self.status, OutboxStatus::Pending | OutboxStatus::Failed)
            && self.process_after <= Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message() {
        let payload = serde_json::json!({"order_id": "ord-001"});
        let msg = OutboxMessage::new(
            "k1s0.service.order.created.v1",
            "ord-001",
            payload,
        );
        assert_eq!(msg.topic, "k1s0.service.order.created.v1");
        assert_eq!(msg.status, OutboxStatus::Pending);
        assert_eq!(msg.retry_count, 0);
        assert!(msg.is_processable());
    }

    #[test]
    fn test_mark_delivered() {
        let mut msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        msg.mark_delivered();
        assert_eq!(msg.status, OutboxStatus::Delivered);
        assert!(!msg.is_processable());
    }

    #[test]
    fn test_mark_failed_increments_retry() {
        let mut msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        msg.mark_failed("kafka error");
        assert_eq!(msg.retry_count, 1);
        assert_eq!(msg.status, OutboxStatus::Failed);
        assert_eq!(msg.last_error.as_deref(), Some("kafka error"));
    }

    #[test]
    fn test_mark_failed_dead_letter_on_max_retries() {
        let mut msg = OutboxMessage::new("test.topic", "key", serde_json::json!({}));
        msg.max_retries = 3;
        msg.mark_failed("error 1");
        msg.mark_failed("error 2");
        msg.mark_failed("error 3");
        assert_eq!(msg.status, OutboxStatus::DeadLetter);
        assert_eq!(msg.retry_count, 3);
    }

    #[test]
    fn test_status_serialization() {
        let status = OutboxStatus::Delivered;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"DELIVERED\"");

        let deserialized: OutboxStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, OutboxStatus::Delivered);
    }
}
