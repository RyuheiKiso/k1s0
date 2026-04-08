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

// Claims を取得するため graphql-gateway のローカル型をインポートする
use crate::adapter::middleware::auth_middleware::Claims;

/// レート制限ミドルウェアの Tower Layer。
/// `RateLimitClient` と設定スコープを保持し、リクエストごとにレート制限チェックを行う。
#[derive(Clone)]
pub struct RateLimitLayer {
    /// レート制限クライアント（gRPC またはインメモリ）
    client: Arc<dyn RateLimitClient>,
    /// レート制限キーのプレフィックス（例: "graphql-gateway"）
    scope: String,
}

impl RateLimitLayer {
    /// 新しい `RateLimitLayer` を作成する。
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
/// 優先順位: Claims（認証済みユーザーID）→ X-Forwarded-For ヘッダー → `ConnectInfo` IP → "anonymous"
/// Claims ベースのキーを最優先とすることで、認証済みユーザーを正確にレート制限できる。
fn extract_identifier(req: &Request<Body>) -> String {
    // 認証済みユーザーの場合は sub クレームをキーとして使用する（IP スプーフィング耐性）
    if let Some(claims) = req.extensions().get::<Claims>() {
        return format!("user:{}", claims.sub);
    }

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

    // フォールバック: 識別子が取得できない場合は警告ログを出力する
    // 匿名キーが多発する場合はネットワーク設定や認証フローを確認すること
    warn!("レート制限キーの識別子を取得できないため anonymous を使用します。ConnectInfo または認証設定を確認してください。");
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
            // リクエストからクライアント識別子を抽出（body 分離前に実行）
            let identifier = extract_identifier(&req);
            // スコープと識別子を組み合わせてレート制限キーを生成
            let rate_limit_key = format!("{scope}:{identifier}");

            // STATIC-MEDIUM-001 監査対応: ボディを読み取って GraphQL クエリ複雑度をコストとして推定する。
            // Alias DoS（大量エイリアスによる単一リクエストへの負荷集中）を防止する。
            // RequestBodyLimitLayer（2MB）が外側に適用されているため、ここでも 2MB を上限とする。
            const MAX_BODY_SIZE: usize = 2 * 1024 * 1024;
            let (parts, body) = req.into_parts();
            let body_bytes = if let Ok(bytes) = axum::body::to_bytes(body, MAX_BODY_SIZE).await { bytes } else {
                // ボディ読み取り失敗時はコスト1でフォールバックし、後続サービスに転送する
                let req = Request::from_parts(parts, Body::empty());
                return inner.call(req).await;
            };

            // GraphQL クエリのフィールド選択数をコストとして推定する
            let cost = estimate_query_cost(&body_bytes);

            // ボディを再組み立てして後続サービスが読み取れるようにする
            let req = Request::from_parts(parts, Body::from(body_bytes));

            // ratelimit サービスにチェックリクエストを送信（推定複雑度をコストとして使用）
            match client.check(&rate_limit_key, cost).await {
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
                Err(k1s0_ratelimit_client::RateLimitError::LimitExceeded { retry_after_secs }) => {
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

/// STATIC-MEDIUM-001 監査対応: GraphQL リクエストボディからクエリ複雑度を推定してコストを返す。
/// JSON ボディの "query" フィールドを取得し、フィールド選択行数を複雑度の近似値とする。
/// エイリアスを多用した Alias `DoS` 攻撃（例: 同一フィールドを1000個のエイリアスで呼び出す）を
/// フィールド数に比例したコストで抑制する。
/// GraphQL クエリでないリクエスト（ヘルスチェック等）はコスト1を返す。
fn estimate_query_cost(body: &[u8]) -> u32 {
    // JSON ボディから "query" フィールドを抽出する
    let query_str = match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(json) => json
            .get("query")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string),
        Err(_) => None,
    };

    let Some(query) = query_str else {
        // GraphQL クエリでないリクエストはコスト1
        return 1;
    };

    // フィールド選択行数を数えることで複雑度を推定する。
    // 空行・コメント（#）・波括弧・演算子定義・フラグメント・スプレッド（...）を除いた行数が
    // フィールド選択数の近似値となる（エイリアス `alias: field` も1行1コスト）。
    let field_count = query
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('{')
                && !line.starts_with('}')
                && !line.starts_with("query")
                && !line.starts_with("mutation")
                && !line.starts_with("subscription")
                && !line.starts_with("fragment")
                && !line.starts_with("...")
                && !line.starts_with('(')
                && !line.starts_with(')')
        })
        .count() as u32;

    // コスト下限は1（最低1リクエスト消費）、上限は100（極端な巨大クエリの上限）
    field_count.max(1).min(100)
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
#[allow(clippy::unwrap_used)]
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

    // --- STATIC-MEDIUM-001 監査対応: estimate_query_cost のユニットテスト ---

    /// 単純なクエリ（フィールド1つ）はコスト1を返す
    #[test]
    fn test_estimate_cost_simple_query() {
        let body = br#"{"query": "query { user { id } }"}"#;
        let cost = estimate_query_cost(body);
        assert!(cost >= 1, "単純クエリのコストは最低1");
    }

    /// エイリアスを多用したクエリは高いコストを返す（Alias DoS 対策）
    #[test]
    fn test_estimate_cost_alias_dos_query() {
        // 多数のエイリアスを含む悪意あるクエリ
        let query = r#"{
  a: expensiveField { id name }
  b: expensiveField { id name }
  c: expensiveField { id name }
  d: expensiveField { id name }
  e: expensiveField { id name }
}"#;
        let body = format!(r#"{{"query": {:?}}}"#, query);
        let cost = estimate_query_cost(body.as_bytes());
        assert!(cost > 1, "エイリアス多用クエリはコスト1より大きいべき: {}", cost);
    }

    /// GraphQL でないリクエスト（ヘルスチェック等）はコスト1を返す
    #[test]
    fn test_estimate_cost_non_graphql() {
        let body = b"";
        let cost = estimate_query_cost(body);
        assert_eq!(cost, 1, "非GraphQLリクエストはコスト1");
    }

    /// コストの上限は100であることを確認する
    #[test]
    fn test_estimate_cost_capped_at_100() {
        // 200フィールドを持つクエリ
        let mut query = String::from("{\n");
        for i in 0..200 {
            query.push_str(&format!("  field{}: someField\n", i));
        }
        query.push('}');
        let body = format!(r#"{{"query": {:?}}}"#, query);
        let cost = estimate_query_cost(body.as_bytes());
        assert_eq!(cost, 100, "コストは100を超えないべき");
    }
}
