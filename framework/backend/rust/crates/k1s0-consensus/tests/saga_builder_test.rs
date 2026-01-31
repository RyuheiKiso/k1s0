//! Saga ビルダーの統合テスト。

use std::sync::Arc;

use async_trait::async_trait;
use k1s0_consensus::saga::{
    BackoffStrategy, RetryPolicy, SagaBuilder, SagaStatus, SagaStep, SagaStepError,
};
use k1s0_consensus::saga::orchestrator::InMemorySagaOrchestrator;

struct AddStep {
    name: &'static str,
    value: i64,
}

#[async_trait]
impl SagaStep for AddStep {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn execute(
        &self,
        context: &serde_json::Value,
    ) -> Result<serde_json::Value, SagaStepError> {
        let current = context
            .get("total")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        Ok(serde_json::json!({ "total": current + self.value }))
    }

    async fn compensate(
        &self,
        _context: &serde_json::Value,
    ) -> Result<(), SagaStepError> {
        Ok(())
    }
}

struct FailingStep;

#[async_trait]
impl SagaStep for FailingStep {
    fn name(&self) -> &'static str {
        "failing-step"
    }

    async fn execute(
        &self,
        _context: &serde_json::Value,
    ) -> Result<serde_json::Value, SagaStepError> {
        Err(SagaStepError::NonRetryable("intentional failure".into()))
    }

    async fn compensate(
        &self,
        _context: &serde_json::Value,
    ) -> Result<(), SagaStepError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_saga_success() {
    let definition = SagaBuilder::new("add-saga")
        .step(Arc::new(AddStep {
            name: "add-10",
            value: 10,
        }))
        .step(Arc::new(AddStep {
            name: "add-20",
            value: 20,
        }))
        .step(Arc::new(AddStep {
            name: "add-30",
            value: 30,
        }))
        .build();

    let orchestrator = InMemorySagaOrchestrator;
    let result = orchestrator
        .execute(&definition, serde_json::json!({ "total": 0 }))
        .await
        .unwrap();

    assert_eq!(result.status, SagaStatus::Completed);
    let output = result.output.unwrap();
    assert_eq!(output["total"], 60);
}

#[tokio::test]
async fn test_saga_compensation() {
    let definition = SagaBuilder::new("fail-saga")
        .step(Arc::new(AddStep {
            name: "add-10",
            value: 10,
        }))
        .step(Arc::new(AddStep {
            name: "add-20",
            value: 20,
        }))
        .step(Arc::new(FailingStep))
        .default_retry(RetryPolicy::no_retry())
        .build();

    let orchestrator = InMemorySagaOrchestrator;
    let result = orchestrator
        .execute(&definition, serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result.status, SagaStatus::Compensated);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_saga_empty_steps() {
    let definition = SagaBuilder::new("empty-saga").build();

    let orchestrator = InMemorySagaOrchestrator;
    let result = orchestrator
        .execute(&definition, serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result.status, SagaStatus::Completed);
}

#[test]
fn test_builder_with_custom_retry() {
    let definition = SagaBuilder::new("custom-retry")
        .step_with_retry(
            Arc::new(AddStep {
                name: "step-1",
                value: 1,
            }),
            RetryPolicy {
                max_retries: 5,
                backoff: BackoffStrategy::Fixed { interval_ms: 50 },
            },
        )
        .step(Arc::new(AddStep {
            name: "step-2",
            value: 2,
        }))
        .build();

    assert_eq!(definition.step_count(), 2);
    assert_eq!(definition.retry_policy_for(0).max_retries, 5);
    assert_eq!(definition.retry_policy_for(1).max_retries, 3); // default
}

#[test]
fn test_backoff_strategy_calculations() {
    let fixed = BackoffStrategy::Fixed { interval_ms: 100 };
    assert_eq!(fixed.delay_ms(0), 100);
    assert_eq!(fixed.delay_ms(10), 100);

    let exp = BackoffStrategy::Exponential {
        base_ms: 100,
        max_ms: 3200,
    };
    assert_eq!(exp.delay_ms(0), 100);
    assert_eq!(exp.delay_ms(1), 200);
    assert_eq!(exp.delay_ms(2), 400);
    assert_eq!(exp.delay_ms(3), 800);
    assert_eq!(exp.delay_ms(4), 1600);
    assert_eq!(exp.delay_ms(5), 3200); // capped
    assert_eq!(exp.delay_ms(10), 3200); // capped
}
