use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

impl GraphQlQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            variables: None,
            operation_name: None,
        }
    }

    #[must_use] 
    pub fn variables(mut self, variables: serde_json::Value) -> Self {
        self.variables = Some(variables);
        self
    }

    pub fn operation_name(mut self, name: impl Into<String>) -> Self {
        self.operation_name = Some(name.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ErrorLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // クエリビルダーで全フィールド（クエリ文字列・変数・オペレーション名）を設定できることを確認する。
    #[test]
    fn test_query_builder() {
        let q = GraphQlQuery::new("{ users { id name } }")
            .variables(serde_json::json!({"limit": 10}))
            .operation_name("GetUsers");

        assert_eq!(q.query, "{ users { id name } }");
        assert!(q.variables.is_some());
        assert_eq!(q.operation_name.unwrap(), "GetUsers");
    }

    // GraphQlQuery::new の最小構成で variables と operation_name が未設定であることを確認する。
    #[test]
    fn test_query_minimal() {
        let q = GraphQlQuery::new("{ health }");
        assert!(q.variables.is_none());
        assert!(q.operation_name.is_none());
    }

    // data フィールドを持つ GraphQL レスポンス JSON が正しくデシリアライズされることを確認する。
    #[test]
    fn test_response_with_data() {
        let json = r#"{"data":{"user":{"id":"1","name":"test"}},"errors":null}"#;
        let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_some());
        assert!(resp.errors.is_none());
    }

    // errors フィールドを持つ GraphQL レスポンス JSON が正しくデシリアライズされることを確認する。
    #[test]
    fn test_response_with_errors() {
        let json = r#"{"data":null,"errors":[{"message":"not found","locations":[{"line":1,"column":3}],"path":["user"]}]}"#;
        let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_none());
        let errors = resp.errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "not found");
        assert_eq!(errors[0].locations.as_ref().unwrap()[0].line, 1);
    }

    // 変数未設定クエリのシリアライズで query フィールドのみが含まれることを確認する。
    #[test]
    fn test_query_serialization() {
        let q = GraphQlQuery::new("{ users { id } }");
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("query"));
        assert!(!json.contains("variables"));
        assert!(!json.contains("operation_name"));
    }
}
