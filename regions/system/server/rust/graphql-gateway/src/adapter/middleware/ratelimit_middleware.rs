//! GraphQL Gateway のレート制限ミドルウェア。
//! ratelimit gRPC サービスと連携して、リクエスト単位のレート制限を実施する。
//! クライアントの IP アドレスまたは認証済みユーザー ID を識別子として使用する。

use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use k1s0_ratelimit_client::RateLimitClient;
use tracing::{info, warn};

/// レート制限ミドルウェアの Tower Layer。
/// RateLimitClient と設定スコープを保持し、リクエストごとにレート制限チェックを行う。
#[derive(Clone)]
pub struct RateLimitLayer {
    /// レート制限クライアント（gRPC またはインメモリ）
    client: Arc<dyn RateLimitClient>,
    /// レート制限キーのプレフィックス（例: "graphql-gateway"）
    scope: String,
}

impl RateLimitLayer {
    /// 新しい RateLimitLayer を作成する。
    /// client: レート制限クライアント実装
    /// scope: キープレフィックス（サービス識別用）
    pub fn new(client: Arc<dyn RateLimitClient>, scope: String) -> Self {
        Self { client, scope }
    }
}

impl<S> tower::Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    /// 内部サービスをラップしてレート制限サービスを生成する
    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            client: self.client.clone(),
            scope: self.scope.clone(),
        }
    }
}

/// レート制限を実施する Tower Service。
/// リクエストの IP またはユーザー ID でレート制限をチェックし、
/// 超過時は HTTP 429 を返す。
#[derive(Clone)]
pub struct RateLimitService<S> {
    /// ラップされた内部サービス
    inner: S,
    /// レート制限クライアント
    client: Arc<dyn RateLimitClient>,
    /// レート制限キーのプレフィックス
    scope: String,
}

/// リクエストからレート制限キーの識別子を抽出する。
/// 優先順位: X-Forwarded-For ヘッダー → リモートアドレス → "anonymous"
fn extract_identifier(req: &Request<Body>) -> String {
    // X-Forwarded-For ヘッダーからクライアント IP を取得（プロキシ経由の場合）
    if let Some(forwarded_for) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        // カンマ区切りの最初の IP を使用
        if let Some(first_ip) = forwarded_for.split(',').next() {
            let trimmed = first_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // ConnectInfo からリモートアドレスを取得（直接接続の場合）
    if let Some(connect_info) = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    {
        return connect_info.0.ip().to_string();
    }

    // フォールバック: 識別子が取得できない場合
    "anonymous".to_string()
}

impl<S> tower::Service<Request<Body>> for RateLimitService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let client = self.client.clone();
        let scope = self.scope.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // リクエストからクライアント識別子を抽出
            let identifier = extract_identifier(&req);
            // スコープと識別子を組み合わせてレート制限キーを生成
            let rate_limit_key = format!("{}:{}", scope, identifier);

            // ratelimit サービスにチェックリクエストを送信（コスト=1）
            match client.check(&rate_limit_key, 1).await {
                Ok(status) => {
                    if !status.allowed {
                        // レート制限超過: HTTP 429 Too Many Requests を返す
                        let retry_after = status.retry_after_secs.unwrap_or(60);
                        warn!(
                            key = %rate_limit_key,
                            retry_after_secs = retry_after,
                            "レート制限超過: リクエストを拒否"
                        );
                        return Ok(rate_limit_exceeded_response(retry_after));
                    }
                    // レート制限内: 残りリクエスト数をログに記録
                    info!(
                        key = %rate_limit_key,
                        remaining = status.remaining,
                        "レート制限チェック通過"
                    );
                }
                Err(k1s0_ratelimit_client::RateLimitError::LimitExceeded {
                    retry_after_secs,
                }) => {
                    // LimitExceeded エラーの場合も HTTP 429 を返す
                    warn!(
                        key = %rate_limit_key,
                        retry_after_secs = retry_after_secs,
                        "レート制限超過（エラー応答）: リクエストを拒否"
                    );
                    return Ok(rate_limit_exceeded_response(retry_after_secs));
                }
                Err(e) => {
                    // ratelimit サービスの障害時はリクエストを通過させる（fail-open）
                    // サービス障害でユーザーリクエストをブロックしない
                    warn!(
                        key = %rate_limit_key,
                        error = %e,
                        "レート制限チェック失敗: fail-open でリクエストを通過"
                    );
                }
            }

            // レート制限チェック通過後、内部サービスにリクエストを転送
            inner.call(req).await
        })
    }
}

/// HTTP 429 Too Many Requests レスポンスを生成する。
/// Retry-After ヘッダーと JSON エラーボディを含む。
fn rate_limit_exceeded_response(retry_after_secs: u64) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "error": {
            "code": "SYS_RATE_LIMIT_EXCEEDED",
            "message": format!(
                "レート制限を超過しました。{}秒後に再試行してください。",
                retry_after_secs
            ),
            "retry_after_secs": retry_after_secs,
            "request_id": request_id,
        }
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", retry_after_secs.to_string().as_str())],
        Json(body),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    /// X-Forwarded-For ヘッダーから最初の IP を抽出するテスト
    #[test]
    fn test_extract_identifier_from_x_forwarded_for() {
        let req = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_identifier(&req), "192.168.1.1");
    }

    /// X-Forwarded-For ヘッダーに単一 IP がある場合のテスト
    #[test]
    fn test_extract_identifier_single_ip() {
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.50")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_identifier(&req), "203.0.113.50");
    }

    /// ヘッダーがない場合にフォールバック値を返すテスト
    #[test]
    fn test_extract_identifier_fallback() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_identifier(&req), "anonymous");
    }

    /// HTTP 429 レスポンスの構造を検証するテスト
    #[tokio::test]
    async fn test_rate_limit_exceeded_response_status() {
        let response = rate_limit_exceeded_response(30);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
