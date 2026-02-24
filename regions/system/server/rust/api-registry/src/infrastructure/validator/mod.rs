pub mod openapi;
pub mod protobuf;

use async_trait::async_trait;

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchemaValidator: Send + Sync {
    async fn validate(&self, content: &str) -> anyhow::Result<Vec<ValidationError>>;
}
