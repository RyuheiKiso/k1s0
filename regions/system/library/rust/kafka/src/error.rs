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

    #[test]
    fn test_connection_failed_display() {
        let err = KafkaError::ConnectionFailed("broker-0:9092 refused".to_string());
        assert_eq!(err.to_string(), "connection failed: broker-0:9092 refused");
    }

    #[test]
    fn test_topic_not_found_display() {
        let err = KafkaError::TopicNotFound("k1s0.system.auth.login.v1".to_string());
        assert_eq!(
            err.to_string(),
            "topic not found: k1s0.system.auth.login.v1"
        );
    }

    #[test]
    fn test_partition_error_display() {
        let err = KafkaError::PartitionError("partition 3 unavailable".to_string());
        assert_eq!(err.to_string(), "partition error: partition 3 unavailable");
    }

    #[test]
    fn test_configuration_error_display() {
        let err = KafkaError::ConfigurationError("missing broker".to_string());
        assert_eq!(err.to_string(), "configuration error: missing broker");
    }

    #[test]
    fn test_timeout_display() {
        let err = KafkaError::Timeout("metadata fetch timed out".to_string());
        assert_eq!(err.to_string(), "timeout: metadata fetch timed out");
    }

    #[test]
    fn test_error_is_debug() {
        let err = KafkaError::PartitionError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("PartitionError"));
    }
}
