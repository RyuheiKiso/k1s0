use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// イベントに付随するメタデータ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// イベント固有の ID。
    pub event_id: Uuid,

    /// イベントが発生した時刻。
    pub occurred_at: DateTime<Utc>,

    /// イベントの発行元サービス名。
    pub source: String,

    /// 分散トレース用の相関 ID（オプション）。
    pub correlation_id: Option<String>,

    /// 因果関係追跡用の ID（オプション）。
    pub causation_id: Option<String>,
}

impl EventMetadata {
    /// 新しいメタデータを生成する。
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            source: source.into(),
            correlation_id: None,
            causation_id: None,
        }
    }

    /// 相関 ID を設定する。
    #[must_use]
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// 因果関係 ID を設定する。
    #[must_use]
    pub fn with_causation_id(mut self, id: impl Into<String>) -> Self {
        self.causation_id = Some(id.into());
        self
    }
}

/// イベント本体とメタデータを格納するエンベロープ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// イベント型の識別子。
    pub event_type: String,

    /// メタデータ。
    pub metadata: EventMetadata,

    /// JSON シリアライズ済みのイベントペイロード。
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// ドメインイベントからエンベロープを作成する。
    ///
    /// # Errors
    ///
    /// イベントの JSON シリアライズに失敗した場合。
    pub fn from_event<E: crate::event::DomainEvent>(
        event: &E,
        source: impl Into<String>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            event_type: event.event_type().to_owned(),
            metadata: EventMetadata::new(source),
            payload: serde_json::to_value(event)?,
        })
    }

    /// メタデータを指定してエンベロープを作成する。
    ///
    /// # Errors
    ///
    /// イベントの JSON シリアライズに失敗した場合。
    pub fn from_event_with_metadata<E: crate::event::DomainEvent>(
        event: &E,
        metadata: EventMetadata,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            event_type: event.event_type().to_owned(),
            metadata,
            payload: serde_json::to_value(event)?,
        })
    }
}
