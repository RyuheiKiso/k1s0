use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Outbox エントリのステータス。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    /// 未処理。
    Pending,
    /// 発行済み。
    Published,
    /// 失敗。
    Failed,
}

/// Outbox テーブルの1行に相当するエントリ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    /// エントリ ID。
    pub id: Uuid,
    /// イベント型。
    pub event_type: String,
    /// JSON シリアライズ済みペイロード。
    pub payload: serde_json::Value,
    /// ステータス。
    pub status: OutboxStatus,
    /// 作成日時。
    pub created_at: DateTime<Utc>,
    /// 発行日時（発行済みの場合）。
    pub published_at: Option<DateTime<Utc>>,
    /// リトライ回数。
    pub retry_count: i32,
    /// 最終エラーメッセージ。
    pub last_error: Option<String>,
}

impl OutboxEntry {
    /// 新しい Pending エントリを作成する。
    #[must_use]
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            payload,
            status: OutboxStatus::Pending,
            created_at: Utc::now(),
            published_at: None,
            retry_count: 0,
            last_error: None,
        }
    }
}
