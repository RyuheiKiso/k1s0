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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer_error_display() {
        let err = MessagingError::ProducerError("kafka broker unreachable".to_string());
        assert!(err.to_string().contains("kafka broker unreachable"));
    }

    #[test]
    fn test_serialization_error_display() {
        let err = MessagingError::SerializationError("invalid json".to_string());
        assert!(err.to_string().contains("invalid json"));
    }

    #[test]
    fn test_consumer_error_display() {
        let err = MessagingError::ConsumerError("offset commit failed".to_string());
        assert_eq!(err.to_string(), "consumer error: offset commit failed");
    }

    #[test]
    fn test_deserialization_error_display() {
        let err = MessagingError::DeserializationError("unexpected field".to_string());
        assert_eq!(err.to_string(), "deserialization error: unexpected field");
    }

    #[test]
    fn test_connection_error_display() {
        let err = MessagingError::ConnectionError("broker unreachable".to_string());
        assert_eq!(err.to_string(), "connection error: broker unreachable");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = MessagingError::TimeoutError("30s exceeded".to_string());
        assert_eq!(err.to_string(), "timeout error: 30s exceeded");
    }

    #[test]
    fn test_producer_error_display_prefix() {
        let err = MessagingError::ProducerError("send failed".to_string());
        assert_eq!(err.to_string(), "producer error: send failed");
    }

    #[test]
    fn test_error_is_debug() {
        let err = MessagingError::ConnectionError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ConnectionError"));
    }
}
