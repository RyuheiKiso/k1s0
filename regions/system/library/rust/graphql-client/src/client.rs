use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::error::ClientError;
use crate::query::{GraphQlQuery, GraphQlResponse};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock(
    type T = serde_json::Value;
))]
pub trait GraphQlClient: Send + Sync {
    async fn execute<T: DeserializeOwned + Send>(
        &self,
        query: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError>;

    async fn execute_mutation<T: DeserializeOwned + Send>(
        &self,
        mutation: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError>;
}

pub struct InMemoryGraphQlClient {
    pub responses: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

impl InMemoryGraphQlClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_response(&self, query: &str, response: serde_json::Value) {
        let mut responses = self.responses.write().await;
        responses.insert(query.to_string(), response);
    }
}

impl Default for InMemoryGraphQlClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphQlClient for InMemoryGraphQlClient {
    async fn execute<T: DeserializeOwned + Send>(
        &self,
        query: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError> {
        let responses = self.responses.read().await;
        match responses.get(&query.query) {
            Some(val) => {
                let data: T = serde_json::from_value(val.clone())
                    .map_err(|e| ClientError::DeserializationError(e.to_string()))?;
                Ok(GraphQlResponse {
                    data: Some(data),
                    errors: None,
                })
            }
            None => Ok(GraphQlResponse {
                data: None,
                errors: None,
            }),
        }
    }

    async fn execute_mutation<T: DeserializeOwned + Send>(
        &self,
        mutation: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError> {
        self.execute(mutation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inmemory_execute() {
        let client = InMemoryGraphQlClient::new();
        client
            .register_response(
                "{ users { id } }",
                serde_json::json!({"users": [{"id": "1"}]}),
            )
            .await;

        let query = GraphQlQuery::new("{ users { id } }");
        let resp: GraphQlResponse<serde_json::Value> = client.execute(query).await.unwrap();
        assert!(resp.data.is_some());
        assert!(resp.errors.is_none());
    }

    #[tokio::test]
    async fn test_inmemory_not_found() {
        let client = InMemoryGraphQlClient::new();
        let query = GraphQlQuery::new("{ unknown }");
        let resp: GraphQlResponse<serde_json::Value> = client.execute(query).await.unwrap();
        assert!(resp.data.is_none());
    }

    #[tokio::test]
    async fn test_inmemory_mutation() {
        let client = InMemoryGraphQlClient::new();
        client
            .register_response(
                "mutation { createUser }",
                serde_json::json!({"id": "new-1"}),
            )
            .await;

        let mutation = GraphQlQuery::new("mutation { createUser }");
        let resp: GraphQlResponse<serde_json::Value> =
            client.execute_mutation(mutation).await.unwrap();
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_error_variants() {
        let err = ClientError::RequestError("timeout".to_string());
        assert!(matches!(err, ClientError::RequestError(_)));

        let err = ClientError::DeserializationError("bad json".to_string());
        assert!(matches!(err, ClientError::DeserializationError(_)));

        let err = ClientError::GraphQlError("server error".to_string());
        assert!(matches!(err, ClientError::GraphQlError(_)));

        let err = ClientError::NotFound("missing".to_string());
        assert!(matches!(err, ClientError::NotFound(_)));
    }

    #[test]
    fn test_default() {
        let client = InMemoryGraphQlClient::default();
        assert!(Arc::strong_count(&client.responses) == 1);
    }
}
