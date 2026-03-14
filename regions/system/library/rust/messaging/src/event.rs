use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// EventMetadata は全 Kafka イベントに付与する共通メタデータ。
/// api/proto/k1s0/system/common/v1/event_metadata.proto に対応する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMetadata {
    /// イベント ID（UUID v4）
    pub event_id: String,
    /// イベント種別（例: "order.created", "auth.login"）
    pub event_type: String,
    /// 発行元サービス名
    pub source: String,
    /// 発行日時
    pub timestamp: DateTime<Utc>,
    /// OpenTelemetry トレース ID
    pub trace_id: Option<String>,
    /// 業務相関 ID
    pub correlation_id: Option<String>,
    /// スキーマバージョン
    pub schema_version: u32,
}

impl EventMetadata {
    /// 新しい EventMetadata を生成する。
    pub fn new(event_type: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            source: source.into(),
            timestamp: Utc::now(),
            trace_id: None,
            correlation_id: None,
            schema_version: 1,
        }
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// timestamp を Unix epoch milliseconds に変換する。
    pub fn to_unix_millis(&self) -> i64 {
        self.timestamp.timestamp_millis()
    }

    /// Unix epoch milliseconds から DateTime<Utc> へ変換する。
    pub fn from_unix_millis(millis: i64) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(millis)
    }
}

/// EventEnvelope はトピック・キー・ペイロードをラップするメッセージエンベロープ。
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    /// 送信先トピック名（例: "k1s0.system.auth.login.v1"）
    pub topic: String,
    /// パーティションキー（例: user_id）
    pub key: String,
    /// JSON シリアライズされたペイロード
    pub payload: Vec<u8>,
    /// メッセージのヘッダー（オプション）
    pub headers: Vec<(String, Vec<u8>)>,
    pub metadata: HashMap<String, String>,
}

impl EventEnvelope {
    /// JSON ペイロードで EventEnvelope を生成する。
    pub fn json<T: Serialize>(
        topic: impl Into<String>,
        key: impl Into<String>,
        payload: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            topic: topic.into(),
            key: key.into(),
            payload: serde_json::to_vec(payload)?,
            headers: Vec::new(),
            metadata: HashMap::new(),
        })
    }

    /// Protobuf バイナリペイロードで EventEnvelope を生成する。
    #[cfg(feature = "protobuf")]
    pub fn protobuf<T: prost::Message>(
        topic: impl Into<String>,
        key: impl Into<String>,
        payload: &T,
    ) -> Self {
        Self {
            topic: topic.into(),
            key: key.into(),
            payload: payload.encode_to_vec(),
            headers: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // EventMetadata::new がイベント種別・発行元・スキーマバージョンを正しく初期化することを確認する。
    #[test]
    fn test_event_metadata_new() {
        let meta = EventMetadata::new("auth.login", "auth-server");
        assert_eq!(meta.event_type, "auth.login");
        assert_eq!(meta.source, "auth-server");
        assert_eq!(meta.schema_version, 1);
        assert!(!meta.event_id.is_empty());
        assert!(meta.trace_id.is_none());
    }

    // with_trace_id と with_correlation_id でトレース ID と相関 ID が設定されることを確認する。
    #[test]
    fn test_event_metadata_with_trace_id() {
        let meta = EventMetadata::new("auth.login", "auth-server")
            .with_trace_id("trace-001")
            .with_correlation_id("corr-001");
        assert_eq!(meta.trace_id.as_deref(), Some("trace-001"));
        assert_eq!(meta.correlation_id.as_deref(), Some("corr-001"));
    }

    // to_unix_millis と from_unix_millis の変換が往復で正しく機能することを確認する。
    #[test]
    fn test_event_metadata_unix_millis_conversion() {
        let meta = EventMetadata::new("auth.login", "auth-server");
        let millis = meta.to_unix_millis();
        let restored = EventMetadata::from_unix_millis(millis).unwrap();
        assert_eq!(restored.timestamp_millis(), millis);
    }

    // EventEnvelope::json が JSON ペイロードを正しくシリアライズしてエンベロープを生成することを確認する。
    #[test]
    fn test_event_envelope_json() {
        let payload = serde_json::json!({"user_id": "user-1", "event": "login"});
        let envelope =
            EventEnvelope::json("k1s0.system.auth.login.v1", "user-1", &payload).unwrap();
        assert_eq!(envelope.topic, "k1s0.system.auth.login.v1");
        assert_eq!(envelope.key, "user-1");
        assert!(!envelope.payload.is_empty());
        assert!(envelope.metadata.is_empty());
    }

    // EventMetadata を JSON シリアライズ・デシリアライズした結果が元のデータと一致することを確認する。
    #[test]
    fn test_event_metadata_serialization_roundtrip() {
        let meta = EventMetadata::new("test.event", "test-service");
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: EventMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, deserialized);
    }

    #[cfg(feature = "protobuf")]
    // Protobuf ペイロードで EventEnvelope が正しく生成されることを確認する。
    #[test]
    fn test_event_envelope_protobuf() {
        #[derive(Clone, PartialEq, prost::Message)]
        struct Sample {
            #[prost(string, tag = "1")]
            user_id: String,
        }

        let payload = Sample {
            user_id: "user-1".to_string(),
        };
        let envelope = EventEnvelope::protobuf("k1s0.system.auth.login.v1", "user-1", &payload);
        assert_eq!(envelope.topic, "k1s0.system.auth.login.v1");
        assert_eq!(envelope.key, "user-1");
        assert!(!envelope.payload.is_empty());
    }
}
