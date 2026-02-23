use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("request error: {0}")]
    RequestError(String),
    #[error("deserialization error: {0}")]
    DeserializationError(String),
    #[error("graphql error: {0}")]
    GraphQlError(String),
    #[error("not found: {0}")]
    NotFound(String),
}
