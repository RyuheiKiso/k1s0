//! Saga 定義と DSL ビルダー。

use std::sync::Arc;

use super::SagaStep;

/// バックオフ戦略。
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// 固定間隔。
    Fixed {
        /// 待機時間（ミリ秒）。
        interval_ms: u64,
    },
    /// 指数バックオフ。
    Exponential {
        /// 基本待機時間（ミリ秒）。
        base_ms: u64,
        /// 最大待機時間（ミリ秒）。
        max_ms: u64,
    },
}

impl BackoffStrategy {
    /// リトライ回数に対する待機時間（ミリ秒）を計算する。
    #[must_use]
    pub fn delay_ms(&self, attempt: u32) -> u64 {
        match self {
            Self::Fixed { interval_ms } => *interval_ms,
            Self::Exponential { base_ms, max_ms } => {
                let delay = base_ms.saturating_mul(2u64.saturating_pow(attempt));
                delay.min(*max_ms)
            }
        }
    }
}

/// リトライポリシー。
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大リトライ回数。
    pub max_retries: u32,
    /// バックオフ戦略。
    pub backoff: BackoffStrategy,
}

impl RetryPolicy {
    /// デフォルトのリトライポリシー（3回、指数バックオフ）。
    #[must_use]
    pub fn default_policy() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                base_ms: 100,
                max_ms: 10_000,
            },
        }
    }

    /// リトライなし。
    #[must_use]
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            backoff: BackoffStrategy::Fixed { interval_ms: 0 },
        }
    }
}

/// Saga 定義内のステップエントリ。
pub struct StepEntry {
    /// ステップ実装。
    pub step: Arc<dyn SagaStep>,
    /// このステップ固有のリトライポリシー（None の場合は Saga デフォルトを使用）。
    pub retry_policy: Option<RetryPolicy>,
}

/// Saga の定義。
pub struct SagaDefinition {
    /// Saga 名。
    pub name: String,
    /// ステップ一覧（実行順）。
    pub steps: Vec<StepEntry>,
    /// デフォルトのリトライポリシー。
    pub default_retry_policy: RetryPolicy,
}

impl SagaDefinition {
    /// ステップのリトライポリシーを取得する（ステップ固有 > デフォルト）。
    #[must_use]
    pub fn retry_policy_for(&self, index: usize) -> &RetryPolicy {
        self.steps
            .get(index)
            .and_then(|e| e.retry_policy.as_ref())
            .unwrap_or(&self.default_retry_policy)
    }

    /// ステップ数を返す。
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// Saga 定義を構築する DSL ビルダー。
pub struct SagaBuilder {
    name: String,
    steps: Vec<StepEntry>,
    default_retry_policy: RetryPolicy,
}

impl SagaBuilder {
    /// 新しいビルダーを作成する。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            default_retry_policy: RetryPolicy::default_policy(),
        }
    }

    /// ステップを追加する。
    #[must_use]
    pub fn step(mut self, step: Arc<dyn SagaStep>) -> Self {
        self.steps.push(StepEntry {
            step,
            retry_policy: None,
        });
        self
    }

    /// リトライポリシー付きでステップを追加する。
    #[must_use]
    pub fn step_with_retry(mut self, step: Arc<dyn SagaStep>, policy: RetryPolicy) -> Self {
        self.steps.push(StepEntry {
            step,
            retry_policy: Some(policy),
        });
        self
    }

    /// デフォルトのリトライポリシーを設定する。
    #[must_use]
    pub fn default_retry(mut self, policy: RetryPolicy) -> Self {
        self.default_retry_policy = policy;
        self
    }

    /// `SagaDefinition` を構築する。
    #[must_use]
    pub fn build(self) -> SagaDefinition {
        SagaDefinition {
            name: self.name,
            steps: self.steps,
            default_retry_policy: self.default_retry_policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_fixed() {
        let backoff = BackoffStrategy::Fixed { interval_ms: 200 };
        assert_eq!(backoff.delay_ms(0), 200);
        assert_eq!(backoff.delay_ms(5), 200);
    }

    #[test]
    fn test_backoff_exponential() {
        let backoff = BackoffStrategy::Exponential {
            base_ms: 100,
            max_ms: 5000,
        };
        assert_eq!(backoff.delay_ms(0), 100); // 100 * 2^0
        assert_eq!(backoff.delay_ms(1), 200); // 100 * 2^1
        assert_eq!(backoff.delay_ms(2), 400); // 100 * 2^2
        assert_eq!(backoff.delay_ms(10), 5000); // capped at max
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default_policy();
        assert_eq!(policy.max_retries, 3);
    }

    #[test]
    fn test_saga_builder() {
        use async_trait::async_trait;

        struct DummyStep;

        #[async_trait]
        impl SagaStep for DummyStep {
            fn name(&self) -> &'static str {
                "dummy"
            }
            async fn execute(
                &self,
                _context: &serde_json::Value,
            ) -> Result<serde_json::Value, super::super::SagaStepError> {
                Ok(serde_json::json!({}))
            }
            async fn compensate(
                &self,
                _context: &serde_json::Value,
            ) -> Result<(), super::super::SagaStepError> {
                Ok(())
            }
        }

        let definition = SagaBuilder::new("test-saga")
            .step(Arc::new(DummyStep))
            .step_with_retry(Arc::new(DummyStep), RetryPolicy::no_retry())
            .build();

        assert_eq!(definition.name, "test-saga");
        assert_eq!(definition.step_count(), 2);
        assert_eq!(definition.retry_policy_for(0).max_retries, 3); // default
        assert_eq!(definition.retry_policy_for(1).max_retries, 0); // custom
    }
}
