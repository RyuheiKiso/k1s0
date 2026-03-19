// HTTP バインディングモジュール。
// reqwest を使用して HTTP リクエストを OutputBinding として実装する。

// 標準ライブラリの HashMap を使用してメタデータを管理する
use std::collections::HashMap;

// async_trait マクロ: async fn を trait 定義で使用可能にする
use async_trait::async_trait;
// reqwest::Client: 非同期 HTTP クライアント
use reqwest::Client;
// RwLock: 非同期対応の読み書きロック（ステータス管理に使用）
use tokio::sync::RwLock;
// info マクロ: 構造化ログ出力に使用する
use tracing::info;

// Component トレイトおよびコンポーネント共通型をインポートする
use k1s0_bb_core::{Component, ComponentError, ComponentStatus};

// バインディングの抽象型とレスポンス型をインポートする
use crate::traits::{BindingResponse, OutputBinding};
// バインディングエラー型をインポートする
use crate::BindingError;

/// HttpOutputBinding は HTTP リクエストを OutputBinding として実装する。
///
/// サポートする operation（HTTP メソッド）: GET, POST, PUT, DELETE, PATCH
///
/// 必須 metadata:
/// - `"url"`: リクエスト先 URL
///
/// 省略可能 metadata:
/// - `"content-type"`: Content-Type ヘッダー（data が非空の場合のデフォルト: "application/octet-stream"）
/// - その他のキーはリクエストヘッダーとして転送される。
///
/// レスポンス metadata には `"status-code"` とレスポンスヘッダーが含まれる。
pub struct HttpOutputBinding {
    // コンポーネントを識別する名前
    name: String,
    // HTTP リクエストを送信するための非同期クライアント
    client: Client,
    // コンポーネントの現在のライフサイクル状態（非同期アクセスのため RwLock で保護）
    status: RwLock<ComponentStatus>,
}

impl HttpOutputBinding {
    /// デフォルトの reqwest::Client を使用して新しいインスタンスを生成する。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            client: Client::new(),
            // 初期状態は未初期化とする
            status: RwLock::new(ComponentStatus::Uninitialized),
        }
    }

    /// カスタム reqwest::Client を注入して新しいインスタンスを生成する。
    /// テスト時にモッククライアントを差し込む用途で使用する。
    pub fn with_client(name: impl Into<String>, client: Client) -> Self {
        Self {
            name: name.into(),
            client,
            // 初期状態は未初期化とする
            status: RwLock::new(ComponentStatus::Uninitialized),
        }
    }
}

// Component トレイトの実装: ライフサイクル管理（init/close/status/metadata）を提供する
#[async_trait]
impl Component for HttpOutputBinding {
    /// コンポーネント名を返す。
    fn name(&self) -> &str {
        &self.name
    }

    /// コンポーネント種別を返す。出力バインディングであることを示す。
    fn component_type(&self) -> &str {
        "binding.output"
    }

    /// コンポーネントを初期化してステータスを Ready に遷移させる。
    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "HttpOutputBinding を初期化しました");
        Ok(())
    }

    /// コンポーネントをクローズしてステータスを Closed に遷移させる。
    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        info!(component = %self.name, "HttpOutputBinding をクローズしました");
        Ok(())
    }

    /// 現在のコンポーネントステータスを返す。
    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    /// コンポーネントのメタデータを返す。
    /// バックエンド種別（http）と方向（output）を示すエントリを含む。
    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "http".to_string());
        meta.insert("direction".to_string(), "output".to_string());
        meta
    }
}

// OutputBinding トレイトの実装: HTTP リクエストの送信ロジックを提供する
#[async_trait]
impl OutputBinding for HttpOutputBinding {
    /// HTTP リクエストを送信して結果を BindingResponse として返す。
    ///
    /// - `operation`: HTTP メソッド文字列（例: "GET", "POST"）
    /// - `data`: リクエストボディ（空の場合はボディなし）
    /// - `metadata`: リクエスト設定（url、content-type、追加ヘッダー）
    async fn invoke(
        &self,
        operation: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<BindingResponse, BindingError> {
        // metadata が None の場合は空の HashMap を使用する
        let meta = metadata.unwrap_or_default();

        // 必須フィールド "url" が存在しない場合はエラーを返す
        let url = meta
            .get("url")
            .ok_or_else(|| BindingError::Invoke(r#"metadata["url"] は必須です"#.to_string()))?;

        // operation 文字列を reqwest::Method に変換する（不正な値はエラー）
        let method = reqwest::Method::from_bytes(operation.as_bytes())
            .map_err(|_| BindingError::UnsupportedOperation(operation.to_string()))?;

        // リクエストビルダーを生成する
        let mut req = self.client.request(method, url);

        // data が非空の場合のみボディと Content-Type ヘッダーを設定する
        if !data.is_empty() {
            // content-type が指定されていない場合は "application/octet-stream" をデフォルトとする
            let ct = meta
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".to_string());
            req = req.header("content-type", ct).body(data.to_vec());
        }

        // "url" と "content-type" を除く metadata エントリをリクエストヘッダーとして追加する
        for (k, v) in &meta {
            if k == "url" || k == "content-type" {
                continue;
            }
            req = req.header(k.as_str(), v.as_str());
        }

        // HTTP リクエストを送信する（ネットワークエラーは Connection エラーに変換）
        let resp = req
            .send()
            .await
            .map_err(|e| BindingError::Connection(e.to_string()))?;

        // レスポンスのステータスコードを取得する
        let status_code = resp.status();

        // レスポンスメタデータを構築する: ステータスコードとレスポンスヘッダーを格納する
        let mut resp_meta = HashMap::new();
        resp_meta.insert("status-code".to_string(), status_code.as_u16().to_string());
        for (k, v) in resp.headers() {
            // UTF-8 に変換可能なヘッダー値のみを格納する
            if let Ok(v_str) = v.to_str() {
                resp_meta.insert(k.as_str().to_string(), v_str.to_string());
            }
        }

        // レスポンスボディを読み取る（読み取りエラーは Read エラーに変換）
        let resp_data = resp
            .bytes()
            .await
            .map_err(|e| BindingError::Read(e.to_string()))?
            .to_vec();

        // 4xx または 5xx レスポンスはエラーとして返す
        if status_code.is_client_error() || status_code.is_server_error() {
            return Err(BindingError::Invoke(format!(
                "HTTP {}: {}",
                status_code,
                String::from_utf8_lossy(&resp_data)
            )));
        }

        // 成功レスポンスをボディとメタデータとともに返す
        Ok(BindingResponse {
            data: resp_data,
            metadata: resp_meta,
        })
    }
}

// テストモジュール: HttpOutputBinding の動作を検証する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 初期化前後のステータス遷移を検証する
    #[tokio::test]
    async fn test_http_binding_init() {
        let binding = HttpOutputBinding::new("test-http");
        assert_eq!(binding.status().await, ComponentStatus::Uninitialized);
        binding.init().await.unwrap();
        assert_eq!(binding.status().await, ComponentStatus::Ready);
    }

    // metadata に "url" が含まれない場合に Invoke エラーが返ることを検証する
    #[tokio::test]
    async fn test_http_binding_missing_url() {
        let binding = HttpOutputBinding::new("test-http");
        let result = binding.invoke("GET", b"", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BindingError::Invoke(_)));
    }

    // 不正な HTTP メソッド文字列を渡した場合に UnsupportedOperation エラーが返ることを検証する
    #[tokio::test]
    async fn test_http_binding_invalid_method() {
        let binding = HttpOutputBinding::new("test-http");
        let mut meta = HashMap::new();
        meta.insert("url".to_string(), "http://example.com".to_string());
        let result = binding.invoke("INVALID METHOD!", b"", Some(meta)).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BindingError::UnsupportedOperation(_)
        ));
    }

    // component_type が "binding.output" を返すことを検証する
    #[test]
    fn test_http_binding_component_type() {
        let binding = HttpOutputBinding::new("test-http");
        assert_eq!(binding.component_type(), "binding.output");
    }

    // metadata が backend と direction を正しく返すことを検証する
    #[test]
    fn test_http_binding_metadata() {
        let binding = HttpOutputBinding::new("test-http");
        let meta = binding.metadata();
        assert_eq!(meta.get("backend").unwrap(), "http");
        assert_eq!(meta.get("direction").unwrap(), "output");
    }
}
