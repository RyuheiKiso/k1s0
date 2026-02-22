use thiserror::Error;

/// SagaError は Saga クライアント操作のエラー型。
#[derive(Debug, Error)]
pub enum SagaError {
    /// HTTP 通信エラー
    #[error("saga network error: {0}")]
    NetworkError(String),

    /// レスポンスのデシリアライズエラー
    #[error("saga deserialize error: {0}")]
    DeserializeError(String),

    /// サーバーが HTTP エラーを返した場合
    #[error("saga API error (status {status_code}): {message}")]
    ApiError { status_code: u16, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_display() {
        let err = SagaError::NetworkError("connection refused".to_string());
        assert_eq!(err.to_string(), "saga network error: connection refused");
    }

    #[test]
    fn test_deserialize_error_display() {
        let err = SagaError::DeserializeError("invalid JSON".to_string());
        assert_eq!(err.to_string(), "saga deserialize error: invalid JSON");
    }

    #[test]
    fn test_api_error_display() {
        let err = SagaError::ApiError {
            status_code: 404,
            message: "saga not found".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "saga API error (status 404): saga not found"
        );
    }
}
