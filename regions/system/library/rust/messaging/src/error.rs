/// MessagingError はメッセージング操作に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("producer error: {0}")]
    ProducerError(String),

    #[error("consumer error: {0}")]
    ConsumerError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("deserialization error: {0}")]
    DeserializationError(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("timeout error: {0}")]
    TimeoutError(String),

    #[error("publish error: {0}")]
    PublishError(String),

    #[error("consume error: {0}")]
    ConsumeError(String),

    #[error("commit error: {0}")]
    CommitError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ProducerError の表示文字列にエラーメッセージが含まれることを確認する。
    #[test]
    fn test_producer_error_display() {
        let err = MessagingError::ProducerError("kafka broker unreachable".to_string());
        assert!(err.to_string().contains("kafka broker unreachable"));
    }

    // SerializationError の表示文字列にエラーメッセージが含まれることを確認する。
    #[test]
    fn test_serialization_error_display() {
        let err = MessagingError::SerializationError("invalid json".to_string());
        assert!(err.to_string().contains("invalid json"));
    }

    // ConsumerError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_consumer_error_display() {
        let err = MessagingError::ConsumerError("offset commit failed".to_string());
        assert_eq!(err.to_string(), "consumer error: offset commit failed");
    }

    // DeserializationError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_deserialization_error_display() {
        let err = MessagingError::DeserializationError("unexpected field".to_string());
        assert_eq!(err.to_string(), "deserialization error: unexpected field");
    }

    // ConnectionError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_connection_error_display() {
        let err = MessagingError::ConnectionError("broker unreachable".to_string());
        assert_eq!(err.to_string(), "connection error: broker unreachable");
    }

    // TimeoutError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_timeout_error_display() {
        let err = MessagingError::TimeoutError("30s exceeded".to_string());
        assert_eq!(err.to_string(), "timeout error: 30s exceeded");
    }

    // ProducerError の表示文字列が "producer error:" プレフィックスを持つことを確認する。
    #[test]
    fn test_producer_error_display_prefix() {
        let err = MessagingError::ProducerError("send failed".to_string());
        assert_eq!(err.to_string(), "producer error: send failed");
    }

    // MessagingError が Debug トレイトを実装しており、デバッグ文字列にバリアント名が含まれることを確認する。
    #[test]
    fn test_error_is_debug() {
        let err = MessagingError::ConnectionError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ConnectionError"));
    }

    // PublishError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_publish_error_display() {
        let err = MessagingError::PublishError("message too large".to_string());
        assert_eq!(err.to_string(), "publish error: message too large");
    }

    // ConsumeError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_consume_error_display() {
        let err = MessagingError::ConsumeError("partition revoked".to_string());
        assert_eq!(err.to_string(), "consume error: partition revoked");
    }

    // CommitError の表示文字列が期待するフォーマットであることを確認する。
    #[test]
    fn test_commit_error_display() {
        let err = MessagingError::CommitError("offset out of range".to_string());
        assert_eq!(err.to_string(), "commit error: offset out of range");
    }

    // すべてのエラーバリアントが std::error::Error トレイトを実装していることを確認する。
    #[test]
    fn test_all_variants_implement_error_trait() {
        fn assert_error<E: std::error::Error>(_: &E) {}

        assert_error(&MessagingError::ProducerError("test".to_string()));
        assert_error(&MessagingError::ConsumerError("test".to_string()));
        assert_error(&MessagingError::SerializationError("test".to_string()));
        assert_error(&MessagingError::DeserializationError("test".to_string()));
        assert_error(&MessagingError::ConnectionError("test".to_string()));
        assert_error(&MessagingError::TimeoutError("test".to_string()));
        assert_error(&MessagingError::PublishError("test".to_string()));
        assert_error(&MessagingError::ConsumeError("test".to_string()));
        assert_error(&MessagingError::CommitError("test".to_string()));
    }

    // 各エラーバリアントの Debug 出力にバリアント名が含まれることを確認する。
    #[test]
    fn test_all_variants_debug_contains_variant_name() {
        let variants: Vec<(&str, MessagingError)> = vec![
            (
                "ProducerError",
                MessagingError::ProducerError("x".to_string()),
            ),
            (
                "ConsumerError",
                MessagingError::ConsumerError("x".to_string()),
            ),
            (
                "SerializationError",
                MessagingError::SerializationError("x".to_string()),
            ),
            (
                "DeserializationError",
                MessagingError::DeserializationError("x".to_string()),
            ),
            (
                "ConnectionError",
                MessagingError::ConnectionError("x".to_string()),
            ),
            (
                "TimeoutError",
                MessagingError::TimeoutError("x".to_string()),
            ),
            (
                "PublishError",
                MessagingError::PublishError("x".to_string()),
            ),
            (
                "ConsumeError",
                MessagingError::ConsumeError("x".to_string()),
            ),
            ("CommitError", MessagingError::CommitError("x".to_string())),
        ];

        for (name, err) in variants {
            let debug = format!("{:?}", err);
            assert!(
                debug.contains(name),
                "Debug for {} should contain variant name",
                name
            );
        }
    }

    // 空文字列のエラーメッセージでも表示フォーマットが正しいことを確認する。
    #[test]
    fn test_error_with_empty_message() {
        let err = MessagingError::ProducerError(String::new());
        assert_eq!(err.to_string(), "producer error: ");
    }
}
