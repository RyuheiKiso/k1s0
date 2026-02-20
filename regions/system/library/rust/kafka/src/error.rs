/// KafkaError は Kafka 操作に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum KafkaError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("topic not found: {0}")]
    TopicNotFound(String),

    #[error("partition error: {0}")]
    PartitionError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("timeout: {0}")]
    Timeout(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        assert!(KafkaError::ConnectionFailed("test".to_string()).to_string().contains("test"));
        assert!(KafkaError::TopicNotFound("my-topic".to_string()).to_string().contains("my-topic"));
    }
}
