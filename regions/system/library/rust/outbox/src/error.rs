/// OutboxError はアウトボックス操作に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("store error: {0}")]
    StoreError(String),

    #[error("publish error: {0}")]
    PublishError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("message not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_error() {
        let err = OutboxError::StoreError("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }
}
