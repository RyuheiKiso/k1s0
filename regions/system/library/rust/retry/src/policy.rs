use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    #[must_use]
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    #[must_use]
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    #[must_use]
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    #[must_use]
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    #[must_use]
    pub fn compute_delay(&self, attempt: u32) -> Duration {
        // HIGH-001 監査対応: u128→f64 は時間計算の精度として許容（天文学的な値では精度が落ちるが実用的には問題なし）
        #[allow(clippy::cast_precision_loss)]
        let initial_ms = self.initial_delay.as_millis() as f64;
        #[allow(clippy::cast_precision_loss)]
        let max_ms = self.max_delay.as_millis() as f64;
        // HIGH-001 監査対応: u32→i32 の安全なキャスト（リトライ回数は i32 の範囲内に収まる想定）
        let attempt_i32 = i32::try_from(attempt).unwrap_or(i32::MAX);
        let base = initial_ms * self.multiplier.powi(attempt_i32);
        let capped = base.min(max_ms);
        let delay_ms = if self.jitter {
            let jitter_range = capped * 0.1;
            capped - jitter_range + (rand::random::<f64>() * jitter_range * 2.0)
        } else {
            capped
        };
        // HIGH-001 監査対応: f64→u64 は非負保証後の安全なキャスト（max(0.0) で負値を除去）
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Duration::from_millis(delay_ms.max(0.0) as u64)
    }
}
