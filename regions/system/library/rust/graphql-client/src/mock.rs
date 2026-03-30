//! MockGraphQlClient: テスト用の GraphQlClient モック実装。
//! `async-trait` + `mockall::automock` のライフタイム非互換を回避するため、
//! `mockall` に依存せず手動でモックを実装する。

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Mutex;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio_stream::Stream;

use crate::client::GraphQlClient;
use crate::error::ClientError;
use crate::query::{GraphQlQuery, GraphQlResponse};

// execute・execute_mutation・subscribe の返却値型エイリアス。
// Value で具象化することで、ジェネリックトレイトのライフタイム非互換を回避する。
type FixedResponse = Result<GraphQlResponse<Value>, ClientError>;

/// テスト用 GraphQlClient モック。
/// serde_json::Value で具象化した返却値を事前に登録し、呼び出し順に消費する。
pub struct MockGraphQlClient {
    // execute メソッドの返却値キュー。Mutex で内部可変性を確保する。
    execute_responses: Mutex<VecDeque<FixedResponse>>,
    // execute_mutation メソッドの返却値キュー。
    execute_mutation_responses: Mutex<VecDeque<FixedResponse>>,
    // subscribe メソッドの返却値キュー。
    subscribe_responses: Mutex<VecDeque<FixedResponse>>,
}

impl MockGraphQlClient {
    /// 空のモックを生成する。
    pub fn new() -> Self {
        Self {
            execute_responses: Mutex::new(VecDeque::new()),
            execute_mutation_responses: Mutex::new(VecDeque::new()),
            subscribe_responses: Mutex::new(VecDeque::new()),
        }
    }

    /// execute が次回呼ばれたときに返す値を登録する。
    /// 複数回呼び出すと FIFO 順に消費される。
    pub fn expect_execute(&self, response: FixedResponse) {
        self.execute_responses
            .lock()
            .expect("execute_responses ロック取得に失敗")
            .push_back(response);
    }

    /// execute_mutation が次回呼ばれたときに返す値を登録する。
    /// 複数回呼び出すと FIFO 順に消費される。
    pub fn expect_execute_mutation(&self, response: FixedResponse) {
        self.execute_mutation_responses
            .lock()
            .expect("execute_mutation_responses ロック取得に失敗")
            .push_back(response);
    }

    /// subscribe が次回呼ばれたときに返す単一イベントを登録する。
    /// ストリームは登録した値を1件だけ emit して終了する。
    pub fn expect_subscribe(&self, response: FixedResponse) {
        self.subscribe_responses
            .lock()
            .expect("subscribe_responses ロック取得に失敗")
            .push_back(response);
    }
}

impl Default for MockGraphQlClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphQlClient for MockGraphQlClient {
    /// キューから次の返却値を取り出し、Value を T にデシリアライズして返す。
    /// キューが空の場合は RequestError を返す。
    async fn execute<T: DeserializeOwned + Send>(
        &self,
        _query: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError> {
        // キューから次の返却値を取り出す
        let response = self
            .execute_responses
            .lock()
            .expect("execute_responses ロック取得に失敗")
            .pop_front()
            .ok_or_else(|| {
                ClientError::RequestError("execute のモック返却値が未設定".to_string())
            })?;

        // Value → T に変換して GraphQlResponse<T> を構築する
        convert_response(response)
    }

    /// キューから次の返却値を取り出し、Value を T にデシリアライズして返す。
    /// キューが空の場合は RequestError を返す。
    async fn execute_mutation<T: DeserializeOwned + Send>(
        &self,
        _mutation: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError> {
        // キューから次の返却値を取り出す
        let response = self
            .execute_mutation_responses
            .lock()
            .expect("execute_mutation_responses ロック取得に失敗")
            .pop_front()
            .ok_or_else(|| {
                ClientError::RequestError("execute_mutation のモック返却値が未設定".to_string())
            })?;

        // Value → T に変換して GraphQlResponse<T> を構築する
        convert_response(response)
    }

    /// キューから次の返却値を取り出し、単一イベントの Stream として返す。
    /// キューが空の場合はエラーを返す。
    async fn subscribe<T: DeserializeOwned + Send + 'static>(
        &self,
        _subscription: GraphQlQuery,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<GraphQlResponse<T>, ClientError>> + Send>>,
        ClientError,
    > {
        // キューから次の返却値を取り出す
        let response = self
            .subscribe_responses
            .lock()
            .expect("subscribe_responses ロック取得に失敗")
            .pop_front()
            .ok_or_else(|| {
                ClientError::RequestError("subscribe のモック返却値が未設定".to_string())
            })?;

        // Value → T に変換してから単一イベントの Stream にラップする
        // tokio_stream::iter でスライスをそのまま Stream に変換する
        let typed_result = convert_response::<T>(response);
        Ok(Box::pin(tokio_stream::iter(std::iter::once(typed_result))))
    }
}

/// GraphQlResponse<Value> を GraphQlResponse<T> に変換するヘルパー関数。
/// data フィールドを serde_json::from_value で T にデシリアライズする。
fn convert_response<T: DeserializeOwned>(
    response: FixedResponse,
) -> Result<GraphQlResponse<T>, ClientError> {
    match response {
        // エラーはそのまま伝播する
        Err(e) => Err(e),
        Ok(resp) => {
            // data が Some の場合は Value を T にデシリアライズする
            let data = match resp.data {
                Some(val) => Some(
                    serde_json::from_value::<T>(val)
                        .map_err(|e| ClientError::DeserializationError(e.to_string()))?,
                ),
                None => None,
            };
            Ok(GraphQlResponse {
                data,
                errors: resp.errors,
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // execute のモック返却値が正しく T にデシリアライズされることを確認する。
    #[tokio::test]
    async fn test_mock_execute() {
        let mock = MockGraphQlClient::new();
        mock.expect_execute(Ok(GraphQlResponse {
            data: Some(serde_json::json!({"id": "1"})),
            errors: None,
        }));

        let query = GraphQlQuery::new("{ user { id } }");
        let resp: GraphQlResponse<Value> = mock.execute(query).await.unwrap();
        assert!(resp.data.is_some());
        assert!(resp.errors.is_none());
    }

    // execute_mutation のモック返却値が正しく T にデシリアライズされることを確認する。
    #[tokio::test]
    async fn test_mock_execute_mutation() {
        let mock = MockGraphQlClient::new();
        mock.expect_execute_mutation(Ok(GraphQlResponse {
            data: Some(serde_json::json!({"created": true})),
            errors: None,
        }));

        let mutation = GraphQlQuery::new("mutation { createUser }");
        let resp: GraphQlResponse<Value> = mock.execute_mutation(mutation).await.unwrap();
        assert!(resp.data.is_some());
    }

    // subscribe のモック返却値が単一イベントのストリームとして取得できることを確認する。
    #[tokio::test]
    async fn test_mock_subscribe() {
        use tokio_stream::StreamExt;

        let mock = MockGraphQlClient::new();
        mock.expect_subscribe(Ok(GraphQlResponse {
            data: Some(serde_json::json!({"event": "created"})),
            errors: None,
        }));

        let subscription = GraphQlQuery::new("subscription { onEvent { event } }");
        let mut stream = mock.subscribe::<Value>(subscription).await.unwrap();
        let event = stream.next().await.unwrap().unwrap();
        assert!(event.data.is_some());
    }

    // モック返却値が未設定の場合に RequestError が返ることを確認する。
    #[tokio::test]
    async fn test_mock_execute_empty_queue() {
        let mock = MockGraphQlClient::new();
        let query = GraphQlQuery::new("{ user }");
        let result: Result<GraphQlResponse<Value>, _> = mock.execute(query).await;
        assert!(matches!(result, Err(ClientError::RequestError(_))));
    }

    // エラーレスポンスがそのまま伝播することを確認する。
    #[tokio::test]
    async fn test_mock_execute_error_propagation() {
        let mock = MockGraphQlClient::new();
        mock.expect_execute(Err(ClientError::GraphQlError("server error".to_string())));

        let query = GraphQlQuery::new("{ user }");
        let result: Result<GraphQlResponse<Value>, _> = mock.execute(query).await;
        assert!(matches!(result, Err(ClientError::GraphQlError(_))));
    }
}
