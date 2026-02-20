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
}
