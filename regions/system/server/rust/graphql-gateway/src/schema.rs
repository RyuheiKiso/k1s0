use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};

#[derive(SimpleObject)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub status: String,
}

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> ServiceInfo {
        ServiceInfo {
            name: "k1s0-graphql-gateway".to_string(),
            version: "0.1.0".to_string(),
            status: "ok".to_string(),
        }
    }

    async fn version(&self) -> String {
        "0.1.0".to_string()
    }
}

#[Object]
impl MutationRoot {
    async fn ping(&self) -> String {
        "pong".to_string()
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_query() {
        let schema = build_schema();
        let result = schema.execute("{ health { name status } }").await;
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn version_query() {
        let schema = build_schema();
        let result = schema.execute("{ version }").await;
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn ping_mutation() {
        let schema = build_schema();
        let result = schema.execute("mutation { ping }").await;
        assert!(result.errors.is_empty());
    }
}
