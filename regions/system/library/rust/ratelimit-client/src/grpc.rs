#[cfg(feature = "grpc")]
mod inner {
    use async_trait::async_trait;

    use crate::client::RateLimitClient;
    use crate::error::RateLimitError;
    use crate::types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

    pub struct GrpcRateLimitClient {
        _server_url: String,
    }

    impl GrpcRateLimitClient {
        pub async fn new(server_url: impl Into<String>) -> Result<Self, RateLimitError> {
            Ok(Self {
                _server_url: server_url.into(),
            })
        }
    }

    #[async_trait]
    impl RateLimitClient for GrpcRateLimitClient {
        async fn check(&self, _key: &str, _cost: u32) -> Result<RateLimitStatus, RateLimitError> {
            Err(RateLimitError::ServerError(
                "gRPC client not yet connected".to_string(),
            ))
        }

        async fn consume(
            &self,
            _key: &str,
            _cost: u32,
        ) -> Result<RateLimitResult, RateLimitError> {
            Err(RateLimitError::ServerError(
                "gRPC client not yet connected".to_string(),
            ))
        }

        async fn get_limit(&self, _key: &str) -> Result<RateLimitPolicy, RateLimitError> {
            Err(RateLimitError::ServerError(
                "gRPC client not yet connected".to_string(),
            ))
        }
    }
}

#[cfg(feature = "grpc")]
pub use inner::GrpcRateLimitClient;
