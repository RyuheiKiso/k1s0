use std::time::Duration;

use serde::{Deserialize, Serialize};

/// WorkflowDefinition はSagaワークフロー定義。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

/// WorkflowStep はワークフローの1ステップを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub service: String,
    pub method: String,
    #[serde(default)]
    pub compensate: Option<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

fn default_timeout_secs() -> u64 {
    30
}

/// RetryConfig はリトライ設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_backoff")]
    pub backoff: String,
    #[serde(default = "default_initial_interval_ms")]
    pub initial_interval_ms: u64,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_backoff() -> String {
    "exponential".to_string()
}

fn default_initial_interval_ms() -> u64 {
    1000
}

impl WorkflowDefinition {
    /// YAMLからワークフロー定義を解析する。
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        let def: Self = serde_yaml::from_str(content)?;
        def.validate()?;
        Ok(def)
    }

    /// ワークフロー定義を検証する。
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("workflow name must not be empty");
        }
        if self.steps.is_empty() {
            anyhow::bail!("workflow must have at least one step");
        }
        for (i, step) in self.steps.iter().enumerate() {
            if step.name.is_empty() {
                anyhow::bail!("step {} name must not be empty", i);
            }
            if step.service.is_empty() {
                anyhow::bail!("step {} service must not be empty", i);
            }
            if step.method.is_empty() {
                anyhow::bail!("step {} method must not be empty", i);
            }
        }
        Ok(())
    }

    /// 指定ステップのタイムアウト期間を返す。
    pub fn timeout_duration(&self, step_idx: usize) -> Duration {
        self.steps
            .get(step_idx)
            .map(|s| Duration::from_secs(s.timeout_secs))
            .unwrap_or_else(|| Duration::from_secs(30))
    }
}

impl RetryConfig {
    /// 指定リトライ回数のバックオフ遅延を計算する。
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.initial_interval_ms;
        let delay_ms = base_ms * 2u64.pow(attempt);
        Duration::from_millis(delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 30
    retry:
      max_attempts: 3
      backoff: exponential
      initial_interval_ms: 1000
  - name: process-payment
    service: payment-service
    method: PaymentService.Charge
    compensate: PaymentService.Refund
    timeout_secs: 60
    retry:
      max_attempts: 2
      backoff: exponential
      initial_interval_ms: 2000
  - name: arrange-shipping
    service: shipping-service
    method: ShippingService.CreateShipment
    compensate: ShippingService.CancelShipment
    timeout_secs: 30
"#;

    #[test]
    fn test_from_yaml() {
        let def = WorkflowDefinition::from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(def.name, "order-fulfillment");
        assert_eq!(def.steps.len(), 3);
        assert_eq!(def.steps[0].name, "reserve-inventory");
        assert_eq!(def.steps[0].service, "inventory-service");
        assert_eq!(def.steps[0].method, "InventoryService.Reserve");
        assert_eq!(
            def.steps[0].compensate.as_deref(),
            Some("InventoryService.Release")
        );
        assert_eq!(def.steps[0].timeout_secs, 30);
        assert_eq!(def.steps[1].timeout_secs, 60);
    }

    #[test]
    fn test_validate_empty_name() {
        let yaml = r#"
name: ""
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        assert!(WorkflowDefinition::from_yaml(yaml).is_err());
    }

    #[test]
    fn test_validate_no_steps() {
        let yaml = r#"
name: empty-workflow
steps: []
"#;
        assert!(WorkflowDefinition::from_yaml(yaml).is_err());
    }

    #[test]
    fn test_timeout_duration() {
        let def = WorkflowDefinition::from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(def.timeout_duration(0), Duration::from_secs(30));
        assert_eq!(def.timeout_duration(1), Duration::from_secs(60));
        assert_eq!(def.timeout_duration(99), Duration::from_secs(30)); // fallback
    }

    #[test]
    fn test_retry_backoff() {
        let retry = RetryConfig {
            max_attempts: 3,
            backoff: "exponential".to_string(),
            initial_interval_ms: 1000,
        };
        assert_eq!(retry.delay_for_attempt(0), Duration::from_millis(1000));
        assert_eq!(retry.delay_for_attempt(1), Duration::from_millis(2000));
        assert_eq!(retry.delay_for_attempt(2), Duration::from_millis(4000));
    }

    #[test]
    fn test_default_timeout() {
        let yaml = r#"
name: simple
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let def = WorkflowDefinition::from_yaml(yaml).unwrap();
        assert_eq!(def.steps[0].timeout_secs, 30); // default
        assert!(def.steps[0].retry.is_none());
        assert!(def.steps[0].compensate.is_none());
    }
}
