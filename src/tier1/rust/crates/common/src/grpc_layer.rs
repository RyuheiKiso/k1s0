// 本ファイルは tonic gRPC Server に挿す tower::Layer 実装。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可 / レート制限 / 観測性 / 監査自動発火」
//
// 役割（Go 側 src/tier1/go/internal/common/runtime.go の grpc.UnaryServerInterceptor
// chain と等価）:
//   tonic の `Server::builder().layer(...)` に渡せる Layer を提供する。
//   Layer は incoming http::Request を見て:
//     1. Authorization header を `Authenticator` で verify
//     2. テナント単位の `RateLimiter` で 1 token 取得を試行
//     3. 観測性 span を生やして latency 計測
//     4. 特権 RPC なら成功 / 失敗を `AuditEmitter` に発火
//   失敗時は gRPC status コード（PermissionDenied / Unauthenticated /
//   ResourceExhausted）を含む gRPC レスポンスをそのまま return する。
//
// 設計上の注意:
//   tonic は内部的に hyper::Body を使うため、本 Layer は hyper の汎用 Service
//   形式（http::Request<B> → http::Response<B>）に対応する。
//   inner service は `Clone` 必須（poll_ready チェック後に async 中で再 use する）。

// 標準同期 / 非同期型。
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

// http の Request / Response 型。
use http::{HeaderMap, Request, Response, StatusCode};

// tower の Layer / Service trait。
use tower::{Layer, Service};

// 共通 module。
use crate::audit::{AuditEmitter, build_record, privileged_rpcs};
use crate::auth::{AuthClaims, Authenticator};
use crate::observability::{RpcCallContext, enter_span};
use crate::ratelimit::RateLimiter;

/// `K1s0Layer` は tonic gRPC Server 用の認証 / 流量 / 観測 / 監査統合 Layer。
#[derive(Clone)]
pub struct K1s0Layer {
    auth: Arc<Authenticator>,
    rate_limiter: Arc<RateLimiter>,
    audit_emitter: Arc<dyn AuditEmitter>,
}

impl K1s0Layer {
    /// 新規 Layer を作成する。
    pub fn new(
        auth: Arc<Authenticator>,
        rate_limiter: Arc<RateLimiter>,
        audit_emitter: Arc<dyn AuditEmitter>,
    ) -> Self {
        Self {
            auth,
            rate_limiter,
            audit_emitter,
        }
    }
}

impl<S> Layer<S> for K1s0Layer {
    type Service = K1s0Service<S>;

    fn layer(&self, inner: S) -> Self::Service {
        K1s0Service {
            inner,
            auth: self.auth.clone(),
            rate_limiter: self.rate_limiter.clone(),
            audit_emitter: self.audit_emitter.clone(),
        }
    }
}

/// `K1s0Service` は inner gRPC service を wrap して chain 実行する。
#[derive(Clone)]
pub struct K1s0Service<S> {
    inner: S,
    auth: Arc<Authenticator>,
    rate_limiter: Arc<RateLimiter>,
    audit_emitter: Arc<dyn AuditEmitter>,
}

/// boxed future（Send + 'static）。
type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for K1s0Service<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static + Default,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = BoxFut<Result<Self::Response, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Service を clone して async block に持ち込む（poll_ready は元 self の状態）。
        let mut inner = self.inner.clone();
        let auth = self.auth.clone();
        let rate_limiter = self.rate_limiter.clone();
        let audit_emitter = self.audit_emitter.clone();

        // gRPC FullMethod は path から取り出す（"/<svc>/<rpc>"）。
        let full_method = req.uri().path().to_string();
        let priv_rpcs = privileged_rpcs();
        let is_privileged = priv_rpcs.contains(full_method.trim_start_matches('/'));
        // Authorization header を文字列として抽出（async に持ち込む）。
        let auth_header = extract_auth_header(req.headers());

        // Liveness / Readiness 用の health probe (kubelet が呼ぶ grpc.health.v1.Health
        // と k1s0.tier1.health.v1.HealthService) は Authorization header を持たない
        // ので、auth verify を skip する。auth=jwks 時に probe が
        // "missing authorization" で fail し pod が再起動ループに陥るのを防ぐ。
        let skip_auth = full_method.starts_with("/grpc.health.")
            || full_method.starts_with("/k1s0.tier1.health.")
            || full_method.starts_with("/grpc.reflection.");

        Box::pin(async move {
            if skip_auth {
                return Ok(inner.call(req).await?);
            }
            // 1) 認証: JWT verify。失敗時は gRPC status を含む応答を即返す。
            let claims = match auth.verify_bearer(auth_header.as_deref()).await {
                Ok(c) => c,
                Err(s) => {
                    if is_privileged {
                        audit_emitter
                            .emit(build_record(
                                &AuthClaims::default(),
                                &full_method,
                                s.code(),
                                "auth",
                            ))
                            .await;
                    }
                    return Ok(grpc_status_response::<ResBody>(s));
                }
            };

            // 2) テナント単位の rate limit。
            if !rate_limiter.try_acquire(&claims.tenant_id).await {
                let s = tonic::Status::resource_exhausted("tier1: rate limit exceeded");
                if is_privileged {
                    audit_emitter
                        .emit(build_record(&claims, &full_method, s.code(), "ratelimit"))
                        .await;
                }
                return Ok(grpc_status_response::<ResBody>(s));
            }

            // 3) 観測性 span。
            let (svc_name, method_name) = split_path(&full_method);
            let span = enter_span(RpcCallContext {
                service: svc_name.to_string(),
                method: method_name.to_string(),
                tenant_id: claims.tenant_id.clone(),
            });

            // claims を request extensions に積んで handler に伝搬する。
            req.extensions_mut().insert(claims.clone());

            // 4) inner handler 呼出。
            let resp = inner.call(req).await?;

            // 5) gRPC status を response trailer から推定する。tonic はステータスを
            //    grpc-status header で返すので、その値を見る。
            let code = grpc_code_from_response(&resp);
            span.finish(code);

            // 6) 監査自動発火（特権 RPC のみ）。
            if is_privileged {
                let resource = format!("k1s0:tenant:{}:rpc:{}", claims.tenant_id, full_method);
                audit_emitter
                    .emit(build_record(&claims, &full_method, code, &resource))
                    .await;
            }

            Ok(resp)
        })
    }
}

/// HTTP request header から `Authorization: Bearer ...` を取り出す。
fn extract_auth_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// `/k1s0.tier1.X.v1.ServiceName/Method` を `(svc, method)` に分割する。
fn split_path(path: &str) -> (&str, &str) {
    let trimmed = path.trim_start_matches('/');
    let mut it = trimmed.splitn(2, '/');
    let svc = it.next().unwrap_or("");
    let rpc = it.next().unwrap_or("");
    (svc, rpc)
}

/// gRPC レスポンスの `grpc-status` ヘッダ（trailer）から `tonic::Code` を取り出す。
/// 値が無ければ Ok 扱い。
fn grpc_code_from_response<B>(resp: &Response<B>) -> tonic::Code {
    let raw = resp
        .headers()
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());
    match raw {
        Some(0) | None => tonic::Code::Ok,
        Some(n) => tonic::Code::from(n),
    }
}

/// `tonic::Status` を gRPC 互換の HTTP レスポンスとして返す。
/// gRPC over HTTP/2 仕様: HTTP status は 200 固定、grpc-status / grpc-message を
/// trailer 形式で返す。tonic 側の `Status::to_http` を流用する。
fn grpc_status_response<ResBody: Default>(status: tonic::Status) -> Response<ResBody> {
    // tonic::Status::to_http は trailers-only な http::Response<empty body> を返す。
    // 内部 body 型は tonic 側固定なため、ここでは trailer header だけ抽出して
    // ResBody::default() で再構築する。
    let src = status.into_http();
    let (parts, _body) = src.into_parts();
    let mut builder = http::Response::builder().status(StatusCode::OK);
    for (k, v) in parts.headers.iter() {
        builder = builder.header(k, v);
    }
    builder.body(ResBody::default()).expect("build response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_handles_normal_grpc_path() {
        let (s, m) = split_path("/k1s0.tier1.audit.v1.AuditService/Record");
        assert_eq!(s, "k1s0.tier1.audit.v1.AuditService");
        assert_eq!(m, "Record");
    }

    #[test]
    fn extract_auth_header_returns_value() {
        let mut h = HeaderMap::new();
        h.insert("authorization", "Bearer abc".parse().unwrap());
        assert_eq!(extract_auth_header(&h).as_deref(), Some("Bearer abc"));
    }

    #[test]
    fn extract_auth_header_returns_none_when_missing() {
        let h = HeaderMap::new();
        assert!(extract_auth_header(&h).is_none());
    }
}
