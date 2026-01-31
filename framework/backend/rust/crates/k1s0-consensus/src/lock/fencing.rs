//! フェンシングトークンバリデータ。
//!
//! フェンシングトークンの単調増加性を検証し、
//! 古いリーダーやロック保持者からの遅延リクエストを拒否する。

use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{ConsensusError, ConsensusResult};

/// フェンシングトークンの単調増加性を検証するバリデータ。
#[derive(Debug)]
pub struct FencingValidator {
    last_seen: AtomicU64,
}

impl FencingValidator {
    /// 新しいバリデータを作成する。
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_seen: AtomicU64::new(0),
        }
    }

    /// 初期トークン値を指定してバリデータを作成する。
    #[must_use]
    pub fn with_initial(token: u64) -> Self {
        Self {
            last_seen: AtomicU64::new(token),
        }
    }

    /// トークンが有効（単調増加）かどうかを検証する。
    ///
    /// 有効な場合は `last_seen` を更新し、`Ok(())` を返す。
    /// 無効な場合は `FenceTokenViolation` エラーを返す。
    ///
    /// # Errors
    ///
    /// トークンが現在の `last_seen` 以下の場合にエラーを返す。
    pub fn validate(&self, token: u64) -> ConsensusResult<()> {
        let current = self.last_seen.load(Ordering::SeqCst);
        if token <= current {
            return Err(ConsensusError::FenceTokenViolation {
                expected: current,
                actual: token,
            });
        }

        // CAS で更新（他スレッドとの競合に対応）
        match self
            .last_seen
            .compare_exchange(current, token, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(_) => Ok(()),
            Err(actual_current) => {
                // 他スレッドが先に更新した場合、再検証
                if token <= actual_current {
                    Err(ConsensusError::FenceTokenViolation {
                        expected: actual_current,
                        actual: token,
                    })
                } else {
                    // リトライ（再帰的に検証）
                    self.validate(token)
                }
            }
        }
    }

    /// 現在の最新トークン値を返す。
    #[must_use]
    pub fn current(&self) -> u64 {
        self.last_seen.load(Ordering::SeqCst)
    }
}

impl Default for FencingValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_increasing_tokens() {
        let v = FencingValidator::new();
        assert!(v.validate(1).is_ok());
        assert!(v.validate(2).is_ok());
        assert!(v.validate(5).is_ok());
        assert_eq!(v.current(), 5);
    }

    #[test]
    fn test_validate_stale_token() {
        let v = FencingValidator::new();
        v.validate(5).unwrap();

        let result = v.validate(3);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConsensusError::FenceTokenViolation { expected, actual } => {
                assert_eq!(expected, 5);
                assert_eq!(actual, 3);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn test_validate_same_token() {
        let v = FencingValidator::new();
        v.validate(5).unwrap();

        let result = v.validate(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_token() {
        let v = FencingValidator::new();
        // 0 は初期値と同じなので無効
        assert!(v.validate(0).is_err());
    }

    #[test]
    fn test_with_initial() {
        let v = FencingValidator::with_initial(100);
        assert!(v.validate(99).is_err());
        assert!(v.validate(101).is_ok());
        assert_eq!(v.current(), 101);
    }
}
