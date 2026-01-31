//! レートリミット設定
//!
//! YAML設定ファイルからのデシリアライズに対応した設定型を提供する。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// トークンバケットのデフォルト容量
pub const DEFAULT_TOKEN_BUCKET_CAPACITY: u64 = 1000;

/// トークンバケットのデフォルト補充レート（トークン/秒）
pub const DEFAULT_TOKEN_BUCKET_REFILL_RATE: f64 = 100.0;

/// スライディングウィンドウのデフォルトサイズ（ミリ秒）
pub const DEFAULT_SLIDING_WINDOW_SIZE_MS: u64 = 60_000;

/// スライディングウィンドウのデフォルト最大リクエスト数
pub const DEFAULT_SLIDING_WINDOW_MAX_REQUESTS: u64 = 600;

fn default_capacity() -> u64 {
    DEFAULT_TOKEN_BUCKET_CAPACITY
}

fn default_refill_rate() -> f64 {
    DEFAULT_TOKEN_BUCKET_REFILL_RATE
}

fn default_window_size_ms() -> u64 {
    DEFAULT_SLIDING_WINDOW_SIZE_MS
}

fn default_max_requests() -> u64 {
    DEFAULT_SLIDING_WINDOW_MAX_REQUESTS
}

/// レートリミット設定
///
/// アルゴリズムごとにタグ付きenumで表現する。
///
/// # YAML設定例
///
/// ```yaml
/// rate_limit:
///   algorithm: token_bucket
///   capacity: 500
///   refill_rate: 50.0
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
pub enum RateLimitConfig {
    /// トークンバケットアルゴリズム
    TokenBucket {
        /// バケット容量
        #[serde(default = "default_capacity")]
        capacity: u64,
        /// 1秒あたりの補充トークン数
        #[serde(default = "default_refill_rate")]
        refill_rate: f64,
    },
    /// スライディングウィンドウアルゴリズム
    SlidingWindow {
        /// ウィンドウサイズ（ミリ秒）
        #[serde(default = "default_window_size_ms")]
        window_size_ms: u64,
        /// ウィンドウ内の最大リクエスト数
        #[serde(default = "default_max_requests")]
        max_requests: u64,
    },
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::TokenBucket {
            capacity: DEFAULT_TOKEN_BUCKET_CAPACITY,
            refill_rate: DEFAULT_TOKEN_BUCKET_REFILL_RATE,
        }
    }
}

impl RateLimitConfig {
    /// デフォルト設定のトークンバケットを生成する
    #[must_use]
    pub fn token_bucket() -> Self {
        Self::TokenBucket {
            capacity: DEFAULT_TOKEN_BUCKET_CAPACITY,
            refill_rate: DEFAULT_TOKEN_BUCKET_REFILL_RATE,
        }
    }

    /// カスタム設定のトークンバケットを生成する
    #[must_use]
    pub fn token_bucket_with(capacity: u64, refill_rate: f64) -> Self {
        Self::TokenBucket {
            capacity,
            refill_rate,
        }
    }

    /// デフォルト設定のスライディングウィンドウを生成する
    #[must_use]
    pub fn sliding_window() -> Self {
        Self::SlidingWindow {
            window_size_ms: DEFAULT_SLIDING_WINDOW_SIZE_MS,
            max_requests: DEFAULT_SLIDING_WINDOW_MAX_REQUESTS,
        }
    }

    /// カスタム設定のスライディングウィンドウを生成する
    #[must_use]
    pub fn sliding_window_with(window_size: Duration, max_requests: u64) -> Self {
        Self::SlidingWindow {
            window_size_ms: u64::try_from(window_size.as_millis()).unwrap_or(u64::MAX),
            max_requests,
        }
    }

    /// スライディングウィンドウのウィンドウサイズを `Duration` として返す
    ///
    /// トークンバケット設定の場合は `None` を返す。
    #[must_use]
    pub fn window_size(&self) -> Option<Duration> {
        match self {
            Self::SlidingWindow { window_size_ms, .. } => {
                Some(Duration::from_millis(*window_size_ms))
            }
            Self::TokenBucket { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_token_bucket() {
        let config = RateLimitConfig::default();
        assert!(matches!(config, RateLimitConfig::TokenBucket { .. }));
    }

    #[test]
    fn test_token_bucket_builder() {
        let config = RateLimitConfig::token_bucket_with(500, 50.0);
        if let RateLimitConfig::TokenBucket {
            capacity,
            refill_rate,
        } = config
        {
            assert_eq!(capacity, 500);
            assert!((refill_rate - 50.0).abs() < f64::EPSILON);
        } else {
            panic!("expected TokenBucket");
        }
    }

    #[test]
    fn test_sliding_window_builder() {
        let config =
            RateLimitConfig::sliding_window_with(Duration::from_secs(30), 100);
        if let RateLimitConfig::SlidingWindow {
            window_size_ms,
            max_requests,
        } = config
        {
            assert_eq!(window_size_ms, 30_000);
            assert_eq!(max_requests, 100);
        } else {
            panic!("expected SlidingWindow");
        }
    }

    #[test]
    fn test_window_size_helper() {
        let tb = RateLimitConfig::token_bucket();
        assert!(tb.window_size().is_none());

        let sw = RateLimitConfig::sliding_window();
        assert_eq!(sw.window_size(), Some(Duration::from_millis(60_000)));
    }

    #[test]
    fn test_serde_roundtrip_token_bucket() {
        let config = RateLimitConfig::token_bucket_with(200, 20.0);
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RateLimitConfig = serde_json::from_str(&json).unwrap();
        if let RateLimitConfig::TokenBucket {
            capacity,
            refill_rate,
        } = parsed
        {
            assert_eq!(capacity, 200);
            assert!((refill_rate - 20.0).abs() < f64::EPSILON);
        } else {
            panic!("expected TokenBucket");
        }
    }

    #[test]
    fn test_serde_roundtrip_sliding_window() {
        let config = RateLimitConfig::sliding_window_with(Duration::from_secs(10), 50);
        let json = serde_json::to_string(&config).unwrap();
        let parsed: RateLimitConfig = serde_json::from_str(&json).unwrap();
        if let RateLimitConfig::SlidingWindow {
            window_size_ms,
            max_requests,
        } = parsed
        {
            assert_eq!(window_size_ms, 10_000);
            assert_eq!(max_requests, 50);
        } else {
            panic!("expected SlidingWindow");
        }
    }
}
