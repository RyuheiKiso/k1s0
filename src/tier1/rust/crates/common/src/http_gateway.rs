// 本ファイルは Rust 共通の HTTP/JSON gateway 実装（axum 経由）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」:
//       URL: POST /k1s0/<api>/<rpc>
//       Content-Type: application/json; charset=utf-8
//       JSON: protojson 直列化
//       認証: Authorization: Bearer <jwt>
//       HTTP Status ↔ K1s0Error マッピング:
//         200 → 成功 / 400 → InvalidArgument / 401 → Unauthenticated /
//         403 → PermissionDenied / 404 → NotFound / 409 → AlreadyExists /
//         429 → ResourceExhausted / 503 → Unavailable / 504 → DeadlineExceeded /
//         500 → Internal
//
// 役割（Go 側 src/tier1/go/internal/common/http_gateway.go と等価）:
//   各 Rust Pod が in-process で gRPC handler を持っているため、HTTP/JSON 経路は
//   handler を直接呼ぶ単純なルータとして実装できる。
//   prost 生成型は serde 直接 derive を持たないため、JSON ↔ proto 変換は呼出側
//   （Pod）が `JsonRpc` trait を実装して提供する。gateway は path → handler の
//   登録と、Authenticator / RateLimiter / AuditEmitter の chain を担当する。

// 標準同期。
use std::sync::Arc;

// axum router と handler。
use axum::{
    Router,
    extract::{Json, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
// JSON 値型。
use serde_json::Value as JsonValue;

// 共通 module。
use crate::audit::{AuditEmitter, build_record, outcome_from_code, privileged_rpcs};
use crate::auth::{AuthClaims, Authenticator};
use crate::observability::{RpcCallContext, enter_span};
use crate::ratelimit::RateLimiter;

/// `JsonRpc` は 1 RPC の JSON ↔ proto 変換 + gRPC 実行を担う adapter trait。
///
/// 実装側（Pod）は:
///   1. JSON Value からその RPC の proto Request 型を組み立て、
///   2. 該当 gRPC handler を呼び、
///   3. proto Response を JSON Value に詰め直す。
///
/// 認証 / レートリミット / 監査は gateway 側が共通で行うため、本 trait は純粋な
/// "JSON in / JSON out + tonic::Status" の関心のみを持つ。
///
/// dyn 互換性のため `async_trait` でデシュガーする（trait object として保持する）。
#[async_trait::async_trait]
pub trait JsonRpc: Send + Sync + 'static {
    /// この RPC の URL 段階での識別子（例: "audit/record"）。
    /// `<api>/<rpc>` 形式の小文字 path 部。
    fn route(&self) -> &'static str;

    /// gRPC FullMethod（例: "/k1s0.tier1.audit.v1.AuditService/Record"）。
    /// observability span / audit action に使う。
    fn full_method(&self) -> &'static str;

    /// JSON Value を proto Request に変換し、認証済 claims を context として
    /// gRPC handler に渡し、proto Response を JSON Value で返す。
    async fn invoke(
        &self,
        claims: &AuthClaims,
        body: JsonValue,
    ) -> Result<JsonValue, tonic::Status>;
}

/// HTTP gateway の構成。
pub struct HttpGateway {
    /// 認証器。
    auth: Arc<Authenticator>,
    /// レートリミット器。
    rate_limiter: Arc<RateLimiter>,
    /// 監査発火器。
    audit_emitter: Arc<dyn AuditEmitter>,
    /// 登録済 RPC（route → impl）。
    handlers: Vec<Arc<dyn JsonRpc>>,
}

impl HttpGateway {
    /// 新規 gateway。
    pub fn new(
        auth: Arc<Authenticator>,
        rate_limiter: Arc<RateLimiter>,
        audit_emitter: Arc<dyn AuditEmitter>,
    ) -> Self {
        Self {
            auth,
            rate_limiter,
            audit_emitter,
            handlers: Vec::new(),
        }
    }

    /// RPC を登録する（builder 風）。
    pub fn register(mut self, rpc: Arc<dyn JsonRpc>) -> Self {
        self.handlers.push(rpc);
        self
    }

    /// axum `Router` を構築する。
    /// 非 POST method は `handle_request` 内で 405 + Allow ヘッダ + JSON body で
    /// 返す（axum default の空 body 405 を避ける、Go 側 http_gateway.go と同じ挙動）。
    pub fn into_router(self) -> Router {
        let shared = Arc::new(SharedState {
            auth: self.auth,
            rate_limiter: self.rate_limiter,
            audit_emitter: self.audit_emitter,
            handlers: self.handlers,
        });
        Router::new()
            .route("/k1s0/:api/:rpc", any(handle_request))
            .with_state(shared)
    }
}

/// `serve` は HttpGateway を `addr` で listen 開始し、stop signal までブロックする。
/// 呼出元（Pod main）は通常これを `tokio::spawn` で別 task 化する。
pub async fn serve(addr: &str, router: Router) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await
}

/// `axum` State として共有される実行時情報。
struct SharedState {
    auth: Arc<Authenticator>,
    rate_limiter: Arc<RateLimiter>,
    audit_emitter: Arc<dyn AuditEmitter>,
    handlers: Vec<Arc<dyn JsonRpc>>,
}

impl SharedState {
    fn lookup(&self, route: &str) -> Option<Arc<dyn JsonRpc>> {
        self.handlers
            .iter()
            .find(|h| h.route() == route)
            .cloned()
    }
}

/// 共通エラー schema（docs §「HTTP Status ↔ K1s0Error マッピング」）。
#[derive(Debug, serde::Serialize)]
struct ErrorBody<'a> {
    /// gRPC code 名（"InvalidArgument" 等）。
    code: &'a str,
    /// 人間可読メッセージ。
    message: &'a str,
}

/// gRPC code → HTTP status 変換（docs マッピング表に従う）。
pub fn http_status_from_grpc(c: tonic::Code) -> StatusCode {
    match c {
        tonic::Code::Ok => StatusCode::OK,
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        tonic::Code::AlreadyExists | tonic::Code::Aborted | tonic::Code::FailedPrecondition => {
            StatusCode::CONFLICT
        }
        tonic::Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        tonic::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        tonic::Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        tonic::Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// axum handler。/k1s0/:api/:rpc にマッチして chain を回す。POST 以外は
/// 405 + Allow: POST + JSON K1s0Error body で reject する（RFC 9110 §15.5.6）。
async fn handle_request(
    State(state): State<Arc<SharedState>>,
    method: Method,
    Path((api, rpc)): Path<(String, String)>,
    headers: HeaderMap,
    body_bytes: axum::body::Bytes,
) -> Response {
    // 非 POST は 405 + Allow + JSON で返す（Go gateway と同等の挙動）。
    if method != Method::POST {
        return write_method_not_allowed("only POST is supported");
    }
    // POST に限り JSON 本体を解釈する。HEAD 等のため、Json extractor は使わず
    // Bytes → JSON parse を自前で行う（Json 不正を InvalidArgument にマップしたい）。
    let body: JsonValue = if body_bytes.is_empty() {
        JsonValue::Object(serde_json::Map::new())
    } else {
        match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(e) => {
                return write_error(
                    tonic::Code::InvalidArgument,
                    &format!("invalid json body: {}", e),
                );
            }
        }
    };
    let route = format!("{}/{}", api, rpc);
    let Some(handler) = state.lookup(&route) else {
        return write_error(tonic::Code::NotFound, "unknown route");
    };

    // 認証: Authorization header から JWT を取り出す。
    let auth_value = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    // 特権 RPC 判定: gRPC 側 K1s0Layer と同じ集合を使う（HTTP / gRPC 経路で挙動を揃える）。
    // full_method は "/<svc>/<method>" 形式なので先頭スラッシュを除いて lookup する。
    let priv_rpcs = privileged_rpcs();
    let is_privileged = priv_rpcs.contains(handler.full_method().trim_start_matches('/'));
    let claims = match state.auth.verify_bearer(auth_value.as_deref()).await {
        Ok(c) => c,
        Err(s) => {
            // 共通規約 §「監査自動発火」: 特権 RPC 失敗のみ audit に記録（gRPC 側と挙動一致）。
            // Audit.Record 自身の認証失敗は DENIED として残したい security 観点もあるが、
            // 仕様としては「特権 RPC 自動発火」表に従う方が一貫しているため統一する。
            if is_privileged {
                state
                    .audit_emitter
                    .emit(build_record(
                        &AuthClaims::default(),
                        handler.full_method(),
                        s.code(),
                        "auth",
                    ))
                    .await;
            }
            return write_error(s.code(), s.message());
        }
    };

    // レートリミット: テナント単位 token bucket。
    if !state.rate_limiter.try_acquire(&claims.tenant_id).await {
        let s = tonic::Status::resource_exhausted("tier1: rate limit exceeded");
        return write_error(s.code(), s.message());
    }

    // 観測性 span を開始。
    let (svc_name, method_name) = split_full_method(handler.full_method());
    let span = enter_span(RpcCallContext {
        service: svc_name.to_string(),
        method: method_name.to_string(),
        tenant_id: claims.tenant_id.clone(),
    });

    // gRPC handler 実行。
    let result = handler.invoke(&claims, body).await;
    let code = result.as_ref().map(|_| tonic::Code::Ok).unwrap_or_else(|s| s.code());
    span.finish(code);

    // 監査: 特権 RPC のみ自動記録（is_privileged は auth 段階で判定済）。
    if is_privileged {
        let resource = format!("k1s0:tenant:{}:rpc:{}", claims.tenant_id, route);
        let mut rec = build_record(&claims, handler.full_method(), code, &resource);
        rec.outcome = outcome_from_code(code).to_string();
        state.audit_emitter.emit(rec).await;
    }

    match result {
        Ok(v) => {
            // 成功時は JSON 本文を返す。
            let mut resp = (StatusCode::OK, Json(v)).into_response();
            resp.headers_mut().insert(
                "content-type",
                "application/json; charset=utf-8".parse().unwrap(),
            );
            resp
        }
        Err(s) => write_error(s.code(), s.message()),
    }
}

/// gRPC FullMethod（"/<svc>/<rpc>"）を service 名と method 名に分割する。
fn split_full_method(full: &str) -> (&str, &str) {
    let trimmed = full.trim_start_matches('/');
    let mut it = trimmed.splitn(2, '/');
    let svc = it.next().unwrap_or("");
    let rpc = it.next().unwrap_or("");
    (svc, rpc)
}

/// エラー応答を生成する。
fn write_error(code: tonic::Code, message: &str) -> Response {
    let status = http_status_from_grpc(code);
    let body = ErrorBody {
        code: code_name(code),
        message,
    };
    let json = serde_json::json!({ "error": body });
    let mut resp = (status, Json(json)).into_response();
    resp.headers_mut().insert(
        "content-type",
        "application/json; charset=utf-8".parse().unwrap(),
    );
    resp
}

/// 405 Method Not Allowed を Allow: POST + JSON K1s0Error で返す。
/// gRPC code 系には 405 が無いため、code 名は HTTP 拡張の "MethodNotAllowed"。
fn write_method_not_allowed(message: &str) -> Response {
    let body = ErrorBody {
        code: "MethodNotAllowed",
        message,
    };
    let json = serde_json::json!({ "error": body });
    let mut resp = (StatusCode::METHOD_NOT_ALLOWED, Json(json)).into_response();
    resp.headers_mut().insert(
        "content-type",
        "application/json; charset=utf-8".parse().unwrap(),
    );
    resp.headers_mut()
        .insert("allow", "POST".parse().unwrap());
    resp
}

/// `tonic::Code` を docs §「エラー型 K1s0Error」表記の文字列に変換する。
fn code_name(c: tonic::Code) -> &'static str {
    match c {
        tonic::Code::Ok => "Ok",
        tonic::Code::Cancelled => "Cancelled",
        tonic::Code::Unknown => "Unknown",
        tonic::Code::InvalidArgument => "InvalidArgument",
        tonic::Code::DeadlineExceeded => "DeadlineExceeded",
        tonic::Code::NotFound => "NotFound",
        tonic::Code::AlreadyExists => "AlreadyExists",
        tonic::Code::PermissionDenied => "PermissionDenied",
        tonic::Code::ResourceExhausted => "ResourceExhausted",
        tonic::Code::FailedPrecondition => "FailedPrecondition",
        tonic::Code::Aborted => "Aborted",
        tonic::Code::OutOfRange => "OutOfRange",
        tonic::Code::Unimplemented => "Unimplemented",
        tonic::Code::Internal => "Internal",
        tonic::Code::Unavailable => "Unavailable",
        tonic::Code::DataLoss => "DataLoss",
        tonic::Code::Unauthenticated => "Unauthenticated",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_status_mapping_follows_docs_table() {
        assert_eq!(http_status_from_grpc(tonic::Code::Ok), StatusCode::OK);
        assert_eq!(http_status_from_grpc(tonic::Code::InvalidArgument), StatusCode::BAD_REQUEST);
        assert_eq!(http_status_from_grpc(tonic::Code::Unauthenticated), StatusCode::UNAUTHORIZED);
        assert_eq!(http_status_from_grpc(tonic::Code::PermissionDenied), StatusCode::FORBIDDEN);
        assert_eq!(http_status_from_grpc(tonic::Code::NotFound), StatusCode::NOT_FOUND);
        assert_eq!(http_status_from_grpc(tonic::Code::AlreadyExists), StatusCode::CONFLICT);
        assert_eq!(http_status_from_grpc(tonic::Code::Aborted), StatusCode::CONFLICT);
        assert_eq!(
            http_status_from_grpc(tonic::Code::FailedPrecondition),
            StatusCode::CONFLICT
        );
        assert_eq!(
            http_status_from_grpc(tonic::Code::ResourceExhausted),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(http_status_from_grpc(tonic::Code::Unavailable), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            http_status_from_grpc(tonic::Code::DeadlineExceeded),
            StatusCode::GATEWAY_TIMEOUT
        );
        assert_eq!(
            http_status_from_grpc(tonic::Code::Internal),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn split_full_method_splits_components() {
        let (s, m) = split_full_method("/k1s0.tier1.audit.v1.AuditService/Record");
        assert_eq!(s, "k1s0.tier1.audit.v1.AuditService");
        assert_eq!(m, "Record");
    }
}
