use k1s0_graphql_client::{
    ClientError, ErrorLocation, GraphQlClient, GraphQlError, GraphQlQuery, GraphQlResponse,
    InMemoryGraphQlClient,
};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

// ---------------------------------------------------------------------------
// GraphQlQuery builder
// ---------------------------------------------------------------------------

// GraphQlQuery::new でクエリ文字列が設定され、variables と operation_name が未設定であることを確認する。
#[test]
fn query_new_sets_query_string() {
    let q = GraphQlQuery::new("{ users { id } }");
    assert_eq!(q.query, "{ users { id } }");
    assert!(q.variables.is_none());
    assert!(q.operation_name.is_none());
}

// variables ビルダーメソッドでクエリに変数が設定できることを確認する。
#[test]
fn query_with_variables() {
    let q = GraphQlQuery::new("query GetUser($id: ID!) { user(id: $id) { name } }")
        .variables(serde_json::json!({"id": "42"}));
    assert!(q.variables.is_some());
    assert_eq!(q.variables.unwrap()["id"], "42");
}

// operation_name ビルダーメソッドでオペレーション名が設定できることを確認する。
#[test]
fn query_with_operation_name() {
    let q = GraphQlQuery::new("query GetUsers { users { id } }").operation_name("GetUsers");
    assert_eq!(q.operation_name.unwrap(), "GetUsers");
}

// ビルダーメソッドをチェーンして全フィールドを一度に設定できることを確認する。
#[test]
fn query_builder_chaining() {
    let q = GraphQlQuery::new("query Q($a: Int) { data(a: $a) { v } }")
        .variables(serde_json::json!({"a": 1}))
        .operation_name("Q");
    assert_eq!(q.query, "query Q($a: Int) { data(a: $a) { v } }");
    assert!(q.variables.is_some());
    assert_eq!(q.operation_name.unwrap(), "Q");
}

// variables と operation_name が未設定の場合、シリアライズで該当フィールドが除外されることを確認する。
#[test]
fn query_serialization_omits_none_fields() {
    let q = GraphQlQuery::new("{ health }");
    let json = serde_json::to_string(&q).unwrap();
    assert!(json.contains("query"));
    assert!(!json.contains("variables"));
    assert!(!json.contains("operation_name"));
}

// variables と operation_name を設定した場合、シリアライズで両フィールドが含まれることを確認する。
#[test]
fn query_serialization_includes_set_fields() {
    let q = GraphQlQuery::new("{ x }")
        .variables(serde_json::json!({"k": "v"}))
        .operation_name("Op");
    let json = serde_json::to_string(&q).unwrap();
    assert!(json.contains("variables"));
    assert!(json.contains("operation_name"));
}

// ---------------------------------------------------------------------------
// GraphQlResponse deserialization
// ---------------------------------------------------------------------------

// data のみを含む JSON レスポンスが正しくデシリアライズされることを確認する。
#[test]
fn response_with_data_only() {
    let json = r#"{"data":{"count":42}}"#;
    let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_some());
    assert_eq!(resp.data.unwrap()["count"], 42);
    assert!(resp.errors.is_none());
}

// errors のみを含む JSON レスポンスが正しくデシリアライズされることを確認する。
#[test]
fn response_with_errors_only() {
    let json = r#"{"data":null,"errors":[{"message":"not authorized"}]}"#;
    let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_none());
    let errors = resp.errors.unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "not authorized");
}

// data と errors の両方を含む JSON レスポンスが正しくデシリアライズされることを確認する。
#[test]
fn response_with_data_and_errors() {
    let json = r#"{"data":{"partial":"ok"},"errors":[{"message":"warn"}]}"#;
    let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
    assert!(resp.data.is_some());
    assert!(resp.errors.is_some());
}

// locations と path を含むエラーレスポンスのフィールドが正しくデシリアライズされることを確認する。
#[test]
fn response_error_with_locations_and_path() {
    let json = r#"{"data":null,"errors":[{"message":"syntax","locations":[{"line":1,"column":5}],"path":["user","name"]}]}"#;
    let resp: GraphQlResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
    let err = &resp.errors.unwrap()[0];
    let loc = &err.locations.as_ref().unwrap()[0];
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 5);
    assert_eq!(err.path.as_ref().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// GraphQlError / ErrorLocation
// ---------------------------------------------------------------------------

// GraphQlError の最小構成（message のみ）が正しく構築されることを確認する。
#[test]
fn graphql_error_minimal() {
    let err = GraphQlError {
        message: "fail".to_string(),
        locations: None,
        path: None,
    };
    assert_eq!(err.message, "fail");
}

// ErrorLocation の line と column フィールドが正しく設定されることを確認する。
#[test]
fn error_location_fields() {
    let loc = ErrorLocation {
        line: 10,
        column: 20,
    };
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 20);
}

// ---------------------------------------------------------------------------
// ClientError variants
// ---------------------------------------------------------------------------

// ClientError::RequestError が正しいバリアントであり、メッセージを含むことを確認する。
#[test]
fn client_error_request() {
    let err = ClientError::RequestError("timeout".to_string());
    assert!(matches!(err, ClientError::RequestError(_)));
    assert!(err.to_string().contains("timeout"));
}

// ClientError::DeserializationError が正しいバリアントであり、メッセージを含むことを確認する。
#[test]
fn client_error_deserialization() {
    let err = ClientError::DeserializationError("bad json".to_string());
    assert!(matches!(err, ClientError::DeserializationError(_)));
    assert!(err.to_string().contains("bad json"));
}

// ClientError::GraphQlError が正しいバリアントに分類されることを確認する。
#[test]
fn client_error_graphql() {
    let err = ClientError::GraphQlError("server error".to_string());
    assert!(matches!(err, ClientError::GraphQlError(_)));
}

// ClientError::NotFound が正しいバリアントであり、メッセージを含むことを確認する。
#[test]
fn client_error_not_found() {
    let err = ClientError::NotFound("resource".to_string());
    assert!(matches!(err, ClientError::NotFound(_)));
    assert!(err.to_string().contains("resource"));
}

// ---------------------------------------------------------------------------
// InMemoryGraphQlClient — execute
// ---------------------------------------------------------------------------

// 登録済みクエリを execute すると対応するデータが返ることを確認する。
#[tokio::test]
async fn execute_registered_query_returns_data() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response(
            "{ users { id } }",
            serde_json::json!({"users": [{"id": "1"}, {"id": "2"}]}),
        )
        .await;

    let query = GraphQlQuery::new("{ users { id } }");
    let resp: GraphQlResponse<serde_json::Value> = client.execute(query).await.unwrap();
    let data = resp.data.unwrap();
    assert_eq!(data["users"].as_array().unwrap().len(), 2);
    assert!(resp.errors.is_none());
}

// 未登録クエリを execute すると data が None で返ることを確認する。
#[tokio::test]
async fn execute_unregistered_query_returns_none_data() {
    let client = InMemoryGraphQlClient::new();
    let query = GraphQlQuery::new("{ unknown_field }");
    let resp: GraphQlResponse<serde_json::Value> = client.execute(query).await.unwrap();
    assert!(resp.data.is_none());
    assert!(resp.errors.is_none());
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: String,
    name: String,
}

// 型付きデシリアライズで登録済みレスポンスを正しく構造体にマッピングできることを確認する。
#[tokio::test]
async fn execute_with_typed_deserialization() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response("{ user }", serde_json::json!({"id": "u1", "name": "Alice"}))
        .await;

    let query = GraphQlQuery::new("{ user }");
    let resp: GraphQlResponse<User> = client.execute(query).await.unwrap();
    let user = resp.data.unwrap();
    assert_eq!(user.id, "u1");
    assert_eq!(user.name, "Alice");
}

// レスポンスと型が一致しない場合に DeserializationError が返ることを確認する。
#[tokio::test]
async fn execute_type_mismatch_returns_error() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response("{ data }", serde_json::json!("not an object"))
        .await;

    #[derive(Debug, Deserialize)]
    struct Complex {
        _field: Vec<i32>,
    }

    let query = GraphQlQuery::new("{ data }");
    let result: Result<GraphQlResponse<Complex>, ClientError> = client.execute(query).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ClientError::DeserializationError(_)
    ));
}

// ---------------------------------------------------------------------------
// InMemoryGraphQlClient — execute_mutation
// ---------------------------------------------------------------------------

// 登録済みミューテーションを execute_mutation すると対応するデータが返ることを確認する。
#[tokio::test]
async fn execute_mutation_returns_data() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response(
            "mutation CreateUser { createUser { id } }",
            serde_json::json!({"createUser": {"id": "new-1"}}),
        )
        .await;

    let mutation = GraphQlQuery::new("mutation CreateUser { createUser { id } }");
    let resp: GraphQlResponse<serde_json::Value> = client.execute_mutation(mutation).await.unwrap();
    assert!(resp.data.is_some());
    assert_eq!(resp.data.unwrap()["createUser"]["id"], "new-1");
}

// execute_mutation が execute と同じロジックで動作することを確認する。
#[tokio::test]
async fn execute_mutation_delegates_to_execute() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response("mutation M", serde_json::json!({"ok": true}))
        .await;

    let q = GraphQlQuery::new("mutation M");
    let resp: GraphQlResponse<serde_json::Value> = client.execute_mutation(q).await.unwrap();
    assert_eq!(resp.data.unwrap()["ok"], true);
}

// ---------------------------------------------------------------------------
// InMemoryGraphQlClient — subscribe
// ---------------------------------------------------------------------------

// 登録済みサブスクリプションイベントがストリームから順番に取得できることを確認する。
#[tokio::test]
async fn subscribe_streams_registered_events() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_subscription_events(
            "OnMessage",
            vec![
                serde_json::json!({"text": "hello"}),
                serde_json::json!({"text": "world"}),
                serde_json::json!({"text": "!"}),
            ],
        )
        .await;

    let sub = GraphQlQuery::new("subscription { messages { text } }").operation_name("OnMessage");
    let mut stream = client.subscribe::<serde_json::Value>(sub).await.unwrap();

    let mut messages = vec![];
    while let Some(result) = stream.next().await {
        let resp = result.unwrap();
        messages.push(resp.data.unwrap()["text"].as_str().unwrap().to_string());
    }
    assert_eq!(messages, vec!["hello", "world", "!"]);
}

// 未登録のサブスクリプションがイベントを返さない空ストリームであることを確認する。
#[tokio::test]
async fn subscribe_unregistered_returns_empty_stream() {
    let client = InMemoryGraphQlClient::new();
    let sub = GraphQlQuery::new("subscription { nope }").operation_name("Nope");
    let mut stream = client.subscribe::<serde_json::Value>(sub).await.unwrap();
    assert!(stream.next().await.is_none());
}

// operation_name が未設定の場合、クエリ文字列をキーとしてサブスクリプションイベントを取得できることを確認する。
#[tokio::test]
async fn subscribe_uses_query_as_key_when_no_operation_name() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_subscription_events("subscription { tick }", vec![serde_json::json!({"n": 1})])
        .await;

    let sub = GraphQlQuery::new("subscription { tick }");
    let mut stream = client.subscribe::<serde_json::Value>(sub).await.unwrap();
    let event = stream.next().await.unwrap().unwrap();
    assert_eq!(event.data.unwrap()["n"], 1);
}

// ---------------------------------------------------------------------------
// Default trait
// ---------------------------------------------------------------------------

// Default トレイトで空のクライアントが正常に生成されることを確認する。
#[test]
fn default_creates_empty_client() {
    let client = InMemoryGraphQlClient::default();
    // Just verify it can be created without panic
    assert!(std::sync::Arc::strong_count(&client.responses) == 1);
}

// ---------------------------------------------------------------------------
// Multiple queries registered independently
// ---------------------------------------------------------------------------

// 複数のクエリが互いに干渉せず独立したレスポンスを返すことを確認する。
#[tokio::test]
async fn multiple_queries_are_independent() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response("{ a }", serde_json::json!({"val": "A"}))
        .await;
    client
        .register_response("{ b }", serde_json::json!({"val": "B"}))
        .await;

    let resp_a: GraphQlResponse<serde_json::Value> =
        client.execute(GraphQlQuery::new("{ a }")).await.unwrap();
    let resp_b: GraphQlResponse<serde_json::Value> =
        client.execute(GraphQlQuery::new("{ b }")).await.unwrap();

    assert_eq!(resp_a.data.unwrap()["val"], "A");
    assert_eq!(resp_b.data.unwrap()["val"], "B");
}

// ---------------------------------------------------------------------------
// Overwriting a registered response
// ---------------------------------------------------------------------------

// 同一クエリキーを再登録すると以前のレスポンスが上書きされることを確認する。
#[tokio::test]
async fn register_response_overwrites_previous() {
    let client = InMemoryGraphQlClient::new();
    client
        .register_response("{ q }", serde_json::json!({"v": 1}))
        .await;
    client
        .register_response("{ q }", serde_json::json!({"v": 2}))
        .await;

    let resp: GraphQlResponse<serde_json::Value> =
        client.execute(GraphQlQuery::new("{ q }")).await.unwrap();
    assert_eq!(resp.data.unwrap()["v"], 2);
}
