use serde::{Deserialize, Serialize};

/// ConsumerConfig は Kafka コンシューマーの設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    /// コンシューマーグループ ID
    pub group_id: String,
    /// サブスクライブするトピックのリスト
    pub topics: Vec<String>,
    /// オートコミット有効フラグ
    #[serde(default = "default_auto_commit")]
    pub auto_commit: bool,
    /// セッションタイムアウト（ミリ秒）
    #[serde(default = "default_session_timeout_ms")]
    pub session_timeout_ms: u64,
}

fn default_auto_commit() -> bool {
    false
}

fn default_session_timeout_ms() -> u64 {
    30000
}

/// ConsumedMessage は Kafka から受信したメッセージを表す。
#[derive(Debug, Clone)]
pub struct ConsumedMessage {
    /// トピック名
    pub topic: String,
    /// パーティション番号
    pub partition: i32,
    /// オフセット
    pub offset: i64,
    /// メッセージキー
    pub key: Option<Vec<u8>>,
    /// メッセージペイロード
    pub payload: Vec<u8>,
}

impl ConsumedMessage {
    /// ペイロードを JSON としてデシリアライズする。
    pub fn deserialize_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.payload)
    }
}

/// EventConsumer は Kafka からのメッセージ受信インターフェース。
#[async_trait::async_trait]
pub trait EventConsumer: Send + Sync {
    /// 次のメッセージを受信する（タイムアウトは実装側で制御）。
    async fn receive(&self) -> Result<ConsumedMessage, crate::error::MessagingError>;

    /// メッセージのオフセットをコミットする。
    async fn commit(&self, msg: &ConsumedMessage) -> Result<(), crate::error::MessagingError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ConsumedMessage のペイロードを JSON としてデシリアライズできることを確認する。
    #[test]
    fn test_consumed_message_deserialize_json() {
        let payload = serde_json::json!({"user_id": "user-1", "event": "login"});
        let msg = ConsumedMessage {
            topic: "k1s0.system.auth.login.v1".to_string(),
            partition: 0,
            offset: 42,
            key: Some(b"user-1".to_vec()),
            payload: serde_json::to_vec(&payload).unwrap(),
        };

        let parsed: serde_json::Value = msg.deserialize_json().unwrap();
        assert_eq!(parsed["user_id"], "user-1");
        assert_eq!(parsed["event"], "login");
    }

    // JSON デシリアライズ時に ConsumerConfig のデフォルト値（auto_commit=false, session_timeout=30000ms）が設定されることを確認する。
    #[test]
    fn test_consumer_config_defaults() {
        let json = r#"{"group_id": "my-group", "topics": ["my-topic"]}"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert!(!cfg.auto_commit);
        assert_eq!(cfg.session_timeout_ms, 30000);
    }

    // キーが None の ConsumedMessage が正しく構築されフィールド値を保持することを確認する。
    #[test]
    fn test_consumed_message_with_none_key() {
        let msg = ConsumedMessage {
            topic: "test.topic".to_string(),
            partition: 1,
            offset: 100,
            key: None,
            payload: b"hello".to_vec(),
        };
        assert_eq!(msg.partition, 1);
        assert_eq!(msg.offset, 100);
        assert!(msg.key.is_none());
    }

    // 複数のトピックを持つ ConsumerConfig が正しくデシリアライズされることを確認する。
    #[test]
    fn test_consumer_config_with_multiple_topics() {
        let json = r#"{"group_id": "my-group", "topics": ["topic-a", "topic-b", "topic-c"]}"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.topics.len(), 3);
        assert_eq!(cfg.group_id, "my-group");
    }

    // 不正な JSON ペイロードのデシリアライズがエラーになることを確認する。
    #[test]
    fn test_consumed_message_deserialize_invalid_json() {
        let msg = ConsumedMessage {
            topic: "test.topic".to_string(),
            partition: 0,
            offset: 0,
            key: None,
            payload: b"not-json".to_vec(),
        };
        let result: Result<serde_json::Value, _> = msg.deserialize_json();
        assert!(result.is_err());
    }

    // auto_commit を明示的に true に指定した場合に ConsumerConfig に反映されることを確認する。
    #[test]
    fn test_consumer_config_auto_commit_override() {
        let json = r#"{"group_id": "grp", "topics": ["t"], "auto_commit": true}"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.auto_commit);
    }

    // session_timeout_ms を明示的に指定した場合に ConsumerConfig に反映されることを確認する。
    #[test]
    fn test_consumer_config_custom_session_timeout() {
        let json = r#"{"group_id": "grp", "topics": ["t"], "session_timeout_ms": 60000}"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.session_timeout_ms, 60000);
    }

    // ConsumerConfig の全フィールドを指定した JSON デシリアライズが正しく動作することを確認する。
    #[test]
    fn test_consumer_config_all_fields_specified() {
        let json = r#"{
            "group_id": "my-consumer-group",
            "topics": ["topic-a", "topic-b"],
            "auto_commit": true,
            "session_timeout_ms": 45000
        }"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.group_id, "my-consumer-group");
        assert_eq!(cfg.topics, vec!["topic-a", "topic-b"]);
        assert!(cfg.auto_commit);
        assert_eq!(cfg.session_timeout_ms, 45000);
    }

    // ConsumerConfig をシリアライズ・デシリアライズしても全フィールドが保持されることを確認する。
    #[test]
    fn test_consumer_config_roundtrip() {
        let cfg = ConsumerConfig {
            group_id: "roundtrip-group".to_string(),
            topics: vec!["t1".to_string(), "t2".to_string()],
            auto_commit: true,
            session_timeout_ms: 15000,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: ConsumerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.group_id, cfg.group_id);
        assert_eq!(restored.topics, cfg.topics);
        assert_eq!(restored.auto_commit, cfg.auto_commit);
        assert_eq!(restored.session_timeout_ms, cfg.session_timeout_ms);
    }

    // ConsumedMessage のペイロードを型付き構造体にデシリアライズできることを確認する。
    #[test]
    fn test_consumed_message_deserialize_typed_struct() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct LoginEvent {
            user_id: String,
            ip_address: String,
        }

        let payload = serde_json::json!({"user_id": "u-1", "ip_address": "192.168.1.1"});
        let msg = ConsumedMessage {
            topic: "auth.login".to_string(),
            partition: 0,
            offset: 10,
            key: Some(b"u-1".to_vec()),
            payload: serde_json::to_vec(&payload).unwrap(),
        };

        let event: LoginEvent = msg.deserialize_json().unwrap();
        assert_eq!(event.user_id, "u-1");
        assert_eq!(event.ip_address, "192.168.1.1");
    }

    // 空の JSON オブジェクトペイロードが正しくデシリアライズされることを確認する。
    #[test]
    fn test_consumed_message_deserialize_empty_json_object() {
        let msg = ConsumedMessage {
            topic: "test".to_string(),
            partition: 0,
            offset: 0,
            key: None,
            payload: b"{}".to_vec(),
        };
        let result: serde_json::Value = msg.deserialize_json().unwrap();
        assert!(result.is_object());
        assert_eq!(result.as_object().unwrap().len(), 0);
    }

    // JSON 配列ペイロードが正しくデシリアライズされることを確認する。
    #[test]
    fn test_consumed_message_deserialize_json_array() {
        let msg = ConsumedMessage {
            topic: "test".to_string(),
            partition: 0,
            offset: 0,
            key: None,
            payload: b"[1, 2, 3]".to_vec(),
        };
        let result: Vec<i32> = msg.deserialize_json().unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    // ConsumedMessage のキーにバイナリデータを設定できることを確認する。
    #[test]
    fn test_consumed_message_with_binary_key() {
        let msg = ConsumedMessage {
            topic: "binary.topic".to_string(),
            partition: 3,
            offset: 999,
            key: Some(vec![0xFF, 0x00, 0xAB]),
            payload: b"data".to_vec(),
        };
        assert_eq!(msg.key.as_ref().unwrap(), &[0xFF, 0x00, 0xAB]);
        assert_eq!(msg.partition, 3);
        assert_eq!(msg.offset, 999);
    }

    // topics が空のリストでも ConsumerConfig が正しくデシリアライズされることを確認する。
    #[test]
    fn test_consumer_config_empty_topics() {
        let json = r#"{"group_id": "grp", "topics": []}"#;
        let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.topics.is_empty());
    }

    // ConsumedMessage の Clone が全フィールドを正しくコピーすることを確認する。
    #[test]
    fn test_consumed_message_clone() {
        let msg = ConsumedMessage {
            topic: "clone.topic".to_string(),
            partition: 2,
            offset: 50,
            key: Some(b"key-data".to_vec()),
            payload: b"payload-data".to_vec(),
        };
        let cloned = msg.clone();
        assert_eq!(cloned.topic, msg.topic);
        assert_eq!(cloned.partition, msg.partition);
        assert_eq!(cloned.offset, msg.offset);
        assert_eq!(cloned.key, msg.key);
        assert_eq!(cloned.payload, msg.payload);
    }
}
