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

    /// ブローカー設定の妥当性を確認する（非同期ヘルスチェック）。
    ///
    /// 実際のブローカー疎通確認は rdkafka 等の具体的なクライアントに委ねる。
    /// このメソッドは設定の論理的妥当性を検証する。
    pub async fn check(&self) -> Result<KafkaHealthStatus, KafkaError> {
        self.check_config()?;
        Ok(KafkaHealthStatus::Healthy)
    }

    /// ブローカーが設定されているかどうか確認する（同期・簡易チェック）。
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_config(brokers: Vec<String>) -> KafkaConfig {
        KafkaConfig {
            brokers,
            security_protocol: "PLAINTEXT".to_string(),
            consumer_group: "test-group".to_string(),
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        }
    }

    // 有効なブローカー設定に対して check_config が Ok を返すことを確認する。
    #[test]
    fn test_check_config_valid() {
        let checker = KafkaHealthChecker::new(make_config(vec!["kafka:9092".to_string()]));
        assert!(checker.check_config().is_ok());
    }

    // ブローカーが空の場合に check_config が ConfigurationError を返すことを確認する。
    #[test]
    fn test_check_config_no_brokers() {
        let checker = KafkaHealthChecker::new(make_config(vec![]));
        let err = checker.check_config().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    // 空文字列のブローカーアドレスが含まれる場合に check_config が ConfigurationError を返すことを確認する。
    #[test]
    fn test_check_config_empty_broker() {
        let checker = KafkaHealthChecker::new(make_config(vec!["".to_string()]));
        let err = checker.check_config().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    // 複数の有効なブローカーアドレスが設定された場合に check_config が Ok を返すことを確認する。
    #[test]
    fn test_check_config_multiple_valid() {
        let checker = KafkaHealthChecker::new(make_config(vec![
            "kafka-0:9092".to_string(),
            "kafka-1:9092".to_string(),
        ]));
        assert!(checker.check_config().is_ok());
    }

    // 有効なブローカー設定に対して非同期の check が Healthy ステータスを返すことを確認する。
    #[tokio::test]
    async fn test_check_async_valid() {
        let checker = KafkaHealthChecker::new(make_config(vec!["kafka:9092".to_string()]));
        let status = checker.check().await.unwrap();
        assert_eq!(status, KafkaHealthStatus::Healthy);
    }

    // ブローカーが空の場合に非同期の check がエラーを返すことを確認する。
    #[tokio::test]
    async fn test_check_async_no_brokers() {
        let checker = KafkaHealthChecker::new(make_config(vec![]));
        let err = checker.check().await.unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }
}
