use async_trait::async_trait;
use crate::checker::HealthCheck;
use crate::error::HealthError;

pub struct HttpHealthCheck {
    name: String,
    url: String,
    timeout_ms: u64,
}

impl HttpHealthCheck {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            timeout_ms: 5000,
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[async_trait]
impl HealthCheck for HttpHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<(), HealthError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| HealthError::CheckFailed(e.to_string()))?;

        let response = client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    HealthError::Timeout(format!("HTTP check timeout: {}", self.url))
                } else {
                    HealthError::CheckFailed(format!("HTTP check failed: {}", e))
                }
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(HealthError::CheckFailed(format!(
                "HTTP {} returned status {}",
                self.url,
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_health_check_new() {
        let check = HttpHealthCheck::new("test", "http://example.com/healthz");
        assert_eq!(check.name(), "test");
        assert_eq!(check.timeout_ms, 5000);
    }

    #[test]
    fn test_http_health_check_with_timeout() {
        let check = HttpHealthCheck::new("test", "http://example.com/healthz")
            .with_timeout_ms(1000);
        assert_eq!(check.timeout_ms, 1000);
    }
}
