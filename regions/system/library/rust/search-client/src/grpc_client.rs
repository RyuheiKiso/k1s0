use async_trait::async_trait;
use reqwest::Client;

use crate::client::SearchClient;
use crate::document::{BulkResult, IndexDocument, IndexMapping, IndexResult};
use crate::error::SearchError;
use crate::query::{SearchQuery, SearchResult};

/// GrpcSearchClient は search-server への HTTP クライアント実装。
/// 名称は gRPC クライアントだが、HTTP/JSON API 経由で search-server と通信する。
pub struct GrpcSearchClient {
    endpoint: String,
    http_client: Client,
}

impl GrpcSearchClient {
    /// 新しい GrpcSearchClient を生成する。
    /// addr は "host:port" または "http://host:port" 形式。
    pub fn new(addr: &str) -> Result<Self, SearchError> {
        let endpoint = normalize_endpoint(addr);
        let http_client = Client::new();
        Ok(Self {
            endpoint,
            http_client,
        })
    }

    /// カスタム reqwest::Client を使う GrpcSearchClient を生成する（テスト用）。
    pub fn with_http_client(addr: &str, http_client: Client) -> Result<Self, SearchError> {
        let endpoint = normalize_endpoint(addr);
        Ok(Self {
            endpoint,
            http_client,
        })
    }
}

/// アドレスを正規化し、スキームを付与して末尾スラッシュを除去する。
fn normalize_endpoint(addr: &str) -> String {
    let endpoint = if !addr.starts_with("http://") && !addr.starts_with("https://") {
        format!("http://{}", addr)
    } else {
        addr.to_string()
    };
    endpoint.trim_end_matches('/').to_string()
}

/// reqwest エラーをタイムアウト・接続エラー・その他に分類して SearchError に変換する。
fn map_reqwest_error(op: &str, e: reqwest::Error) -> SearchError {
    if e.is_timeout() {
        SearchError::Timeout
    } else if e.is_connect() {
        SearchError::ServerError(format!("{} 接続エラー: {}", op, e))
    } else {
        SearchError::ServerError(format!("{} リクエスト失敗: {}", op, e))
    }
}

/// HTTP ステータスコードとレスポンスボディから SearchError を生成する。
fn parse_search_error(status: u16, body: &str) -> SearchError {
    match status {
        404 => SearchError::IndexNotFound(body.to_string()),
        400 => SearchError::InvalidQuery(body.to_string()),
        408 | 504 => SearchError::Timeout,
        _ => SearchError::ServerError(format!("status={}: {}", status, body)),
    }
}

#[async_trait]
impl SearchClient for GrpcSearchClient {
    /// PUT /api/v1/indexes/{name}
    async fn create_index(&self, name: &str, mapping: IndexMapping) -> Result<(), SearchError> {
        let url = format!("{}/api/v1/indexes/{}", self.endpoint, name);

        let body = serde_json::json!({
            "name": name,
            "mapping": mapping,
        });

        let resp = self
            .http_client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_reqwest_error("create_index", e))?;

        let status = resp.status().as_u16();
        if status == 200 || status == 201 || status == 204 {
            return Ok(());
        }

        let resp_body = resp
            .text()
            .await
            .unwrap_or_default();
        Err(parse_search_error(status, &resp_body))
    }

    /// POST /api/v1/indexes/{index}/documents
    async fn index_document(
        &self,
        index: &str,
        doc: IndexDocument,
    ) -> Result<IndexResult, SearchError> {
        let url = format!("{}/api/v1/indexes/{}/documents", self.endpoint, index);

        let resp = self
            .http_client
            .post(&url)
            .json(&doc)
            .send()
            .await
            .map_err(|e| map_reqwest_error("index_document", e))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .text()
            .await
            .unwrap_or_default();

        if status != 200 && status != 201 {
            return Err(parse_search_error(status, &resp_body));
        }

        serde_json::from_str(&resp_body).map_err(|e| {
            SearchError::ServerError(format!("index_document レスポンスのデコード失敗: {}", e))
        })
    }

    /// POST /api/v1/indexes/{index}/documents/_bulk
    async fn bulk_index(
        &self,
        index: &str,
        docs: Vec<IndexDocument>,
    ) -> Result<BulkResult, SearchError> {
        let url = format!(
            "{}/api/v1/indexes/{}/documents/_bulk",
            self.endpoint, index
        );

        let body = serde_json::json!({ "documents": docs });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_reqwest_error("bulk_index", e))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .text()
            .await
            .unwrap_or_default();

        if status != 200 && status != 201 {
            return Err(parse_search_error(status, &resp_body));
        }

        serde_json::from_str(&resp_body).map_err(|e| {
            SearchError::ServerError(format!("bulk_index レスポンスのデコード失敗: {}", e))
        })
    }

    /// POST /api/v1/indexes/{index}/_search
    async fn search(
        &self,
        index: &str,
        query: SearchQuery,
    ) -> Result<SearchResult<serde_json::Value>, SearchError> {
        let url = format!("{}/api/v1/indexes/{}/_search", self.endpoint, index);

        let resp = self
            .http_client
            .post(&url)
            .json(&query)
            .send()
            .await
            .map_err(|e| map_reqwest_error("search", e))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .text()
            .await
            .unwrap_or_default();

        if status != 200 {
            return Err(parse_search_error(status, &resp_body));
        }

        serde_json::from_str(&resp_body).map_err(|e| {
            SearchError::ServerError(format!("search レスポンスのデコード失敗: {}", e))
        })
    }

    /// DELETE /api/v1/indexes/{index}/documents/{id}
    async fn delete_document(&self, index: &str, id: &str) -> Result<(), SearchError> {
        let url = format!(
            "{}/api/v1/indexes/{}/documents/{}",
            self.endpoint, index, id
        );

        let resp = self
            .http_client
            .delete(&url)
            .send()
            .await
            .map_err(|e| map_reqwest_error("delete_document", e))?;

        let status = resp.status().as_u16();
        if status == 200 || status == 204 {
            return Ok(());
        }

        let resp_body = resp
            .text()
            .await
            .unwrap_or_default();
        Err(parse_search_error(status, &resp_body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_endpoint_adds_scheme() {
        assert_eq!(normalize_endpoint("localhost:8080"), "http://localhost:8080");
    }

    #[test]
    fn test_normalize_endpoint_preserves_http() {
        assert_eq!(
            normalize_endpoint("http://localhost:8080"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_normalize_endpoint_preserves_https() {
        assert_eq!(
            normalize_endpoint("https://search.example.com"),
            "https://search.example.com"
        );
    }

    #[test]
    fn test_normalize_endpoint_strips_trailing_slash() {
        assert_eq!(
            normalize_endpoint("http://localhost:8080/"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_parse_search_error_not_found() {
        let err = parse_search_error(404, "index missing");
        assert!(matches!(err, SearchError::IndexNotFound(_)));
    }

    #[test]
    fn test_parse_search_error_bad_request() {
        let err = parse_search_error(400, "bad query syntax");
        assert!(matches!(err, SearchError::InvalidQuery(_)));
    }

    #[test]
    fn test_parse_search_error_timeout_408() {
        let err = parse_search_error(408, "");
        assert!(matches!(err, SearchError::Timeout));
    }

    #[test]
    fn test_parse_search_error_timeout_504() {
        let err = parse_search_error(504, "");
        assert!(matches!(err, SearchError::Timeout));
    }

    #[test]
    fn test_parse_search_error_server_error() {
        let err = parse_search_error(500, "internal");
        assert!(matches!(err, SearchError::ServerError(_)));
    }

    #[test]
    fn test_new_client() {
        let client = GrpcSearchClient::new("localhost:9200");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.endpoint, "http://localhost:9200");
    }

    #[test]
    fn test_with_http_client() {
        let http_client = Client::new();
        let client = GrpcSearchClient::with_http_client("https://search.example.com", http_client);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.endpoint, "https://search.example.com");
    }
}
