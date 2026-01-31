//! コンセンサスエラー定義。

use std::fmt;

/// コンセンサス操作で発生しうるエラー。
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    /// リースが期限切れになった。
    #[error("lease expired for key '{key}': holder '{holder_id}'")]
    LeaseExpired {
        /// リースキー。
        key: String,
        /// リース保持者 ID。
        holder_id: String,
    },

    /// ロック取得がタイムアウトした。
    #[error("lock timeout after {elapsed_ms}ms for resource '{resource}'")]
    LockTimeout {
        /// ロック対象のリソース名。
        resource: String,
        /// 経過時間（ミリ秒）。
        elapsed_ms: u64,
    },

    /// フェンシングトークン違反が検出された。
    #[error("fence token violation: expected > {expected}, got {actual}")]
    FenceTokenViolation {
        /// 期待される最小トークン値。
        expected: u64,
        /// 実際のトークン値。
        actual: u64,
    },

    /// Saga ステップの実行に失敗した。
    #[error("saga '{saga_id}' failed at step '{step_name}': {reason}")]
    SagaFailed {
        /// Saga インスタンス ID。
        saga_id: String,
        /// 失敗したステップ名。
        step_name: String,
        /// 失敗理由。
        reason: String,
    },

    /// Saga の補償処理に失敗した。
    #[error("compensation failed for saga '{saga_id}' at step '{step_name}': {reason}")]
    CompensationFailed {
        /// Saga インスタンス ID。
        saga_id: String,
        /// 補償に失敗したステップ名。
        step_name: String,
        /// 失敗理由。
        reason: String,
    },

    /// デッドレターに送られた。
    #[error("saga '{saga_id}' moved to dead letter: {reason}")]
    DeadLetter {
        /// Saga インスタンス ID。
        saga_id: String,
        /// デッドレターの理由。
        reason: String,
    },

    /// データベースエラー。
    #[error("database error: {0}")]
    Database(String),

    /// Redis エラー。
    #[error("redis error: {0}")]
    Redis(String),

    /// 設定エラー。
    #[error("config error: {0}")]
    Config(String),

    /// シリアライズ/デシリアライズエラー。
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// コンセンサス操作の結果型エイリアス。
pub type ConsensusResult<T> = Result<T, ConsensusError>;

impl ConsensusError {
    /// `k1s0-error` の `ErrorKind` に対応するカテゴリを返す。
    #[must_use]
    pub fn error_kind(&self) -> ErrorCategory {
        match self {
            Self::LeaseExpired { .. }
            | Self::LockTimeout { .. }
            | Self::FenceTokenViolation { .. } => ErrorCategory::Conflict,
            Self::SagaFailed { .. } | Self::CompensationFailed { .. } | Self::DeadLetter { .. } => {
                ErrorCategory::Internal
            }
            Self::Database(_) | Self::Redis(_) => ErrorCategory::DependencyFailure,
            Self::Config(_) => ErrorCategory::InvalidInput,
            Self::Serialization(_) => ErrorCategory::Internal,
        }
    }
}

/// エラーカテゴリ（`k1s0-error` の `ErrorKind` に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 競合エラー。
    Conflict,
    /// 内部エラー。
    Internal,
    /// 依存障害。
    DependencyFailure,
    /// 入力不備。
    InvalidInput,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict => write!(f, "Conflict"),
            Self::Internal => write!(f, "Internal"),
            Self::DependencyFailure => write!(f, "DependencyFailure"),
            Self::InvalidInput => write!(f, "InvalidInput"),
        }
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for ConsensusError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

#[cfg(feature = "redis-backend")]
impl From<redis::RedisError> for ConsensusError {
    fn from(err: redis::RedisError) -> Self {
        Self::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for ConsensusError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ConsensusError::LeaseExpired {
            key: "my-service".into(),
            holder_id: "node-1".into(),
        };
        assert!(err.to_string().contains("my-service"));
        assert!(err.to_string().contains("node-1"));
    }

    #[test]
    fn test_error_category() {
        let err = ConsensusError::LockTimeout {
            resource: "res".into(),
            elapsed_ms: 5000,
        };
        assert_eq!(err.error_kind(), ErrorCategory::Conflict);

        let err = ConsensusError::Database("conn failed".into());
        assert_eq!(err.error_kind(), ErrorCategory::DependencyFailure);
    }
}
