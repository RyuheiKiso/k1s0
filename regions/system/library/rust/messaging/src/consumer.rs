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
}
