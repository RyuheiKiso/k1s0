use crate::config::KafkaConfig;
use crate::error::KafkaError;

/// KafkaHealthStatus は Kafka クラスターのヘルス状態を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum KafkaHealthStatus {
    /// 接続可能・正常
    Healthy,
    /// 接続不可・異常
    Unhealthy(String),
}

/// KafkaHealthChecker は Kafka クラスターのヘルスチェックを提供する。
pub struct KafkaHealthChecker {
    config: KafkaConfig,
}

impl KafkaHealthChecker {
    pub fn new(config: KafkaConfig) -> Self {
        Self { config }
    }

    /// ブローカーが設定されているかどうか確認する（簡易チェック）。
    pub fn check_config(&self) -> Result<(), KafkaError> {
        if self.config.brokers.is_empty() {
            return Err(KafkaError::ConfigurationError(
                "no brokers configured".to_string(),
            ));
        }
        for broker in &self.config.brokers {
            if broker.is_empty() {
                return Err(KafkaError::ConfigurationError(
                    "empty broker address".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(brokers: Vec<String>) -> KafkaConfig {
        KafkaConfig {
            brokers,
            security_protocol: "PLAINTEXT".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        }
    }

    #[test]
    fn test_check_config_valid() {
        let checker = KafkaHealthChecker::new(make_config(vec!["kafka:9092".to_string()]));
        assert!(checker.check_config().is_ok());
    }

    #[test]
    fn test_check_config_no_brokers() {
        let checker = KafkaHealthChecker::new(make_config(vec![]));
        let err = checker.check_config().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    #[test]
    fn test_check_config_empty_broker() {
        let checker = KafkaHealthChecker::new(make_config(vec!["".to_string()]));
        let err = checker.check_config().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    #[test]
    fn test_check_config_multiple_valid() {
        let checker = KafkaHealthChecker::new(make_config(vec![
            "kafka-0:9092".to_string(),
            "kafka-1:9092".to_string(),
        ]));
        assert!(checker.check_config().is_ok());
    }
}
