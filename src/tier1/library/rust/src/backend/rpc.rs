// rpc.rs — k1s0 tier1 Library backend: RPC/Gateway L3 trait（backend 向け）
// backend 向け RPC/Gateway L3 を定義する。
// frontend 向けは src/frontend/rpc.rs で別途定義する（thin wrapper として分離する）。
// 公開 API に OSS 型（tonic::Channel / reqwest::Client / tower::ServiceBuilder 等）を露出しない。
// retry / tracing / auth context 伝播は ServicePolicy 経由で強制する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: リクエスト / レスポンス型のシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: RPC 呼び出しに認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// ServicePolicy: retry / timeout / circuit breaker を強制する
use crate::core::policy::ServicePolicy;
// SpanContext: RPC 呼び出しに tracing コンテキストを伝播する
use crate::core::observability::SpanContext;

// RpcRequest は RPC 呼び出しのリクエストを表す Library 独自型。
// OSS の HTTP Request / gRPC Request を公開 API に露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    // service_name: 呼び出し先サービス名（例: "payment.PaymentService"）
    pub service_name: String,
    // method_name: 呼び出し先メソッド名（例: "ProcessPayment"）
    pub method_name: String,
    // payload_bytes: リクエストペイロード（Protobuf シリアライズ済みバイト列）
    pub payload_bytes: Vec<u8>,
    // headers: 追加ヘッダー（W3C traceparent は別途注入するため含まない）
    pub headers: std::collections::HashMap<String, String>,
}

// RpcResponse は RPC 呼び出しのレスポンスを表す Library 独自型。
// OSS の HTTP Response / gRPC Response を公開 API に露出しない。
#[derive(Debug, Clone)]
pub struct RpcResponse {
    // status_code: ステータスコード（gRPC status code / HTTP status code を Library 独自型に変換）
    pub status_code: u32,
    // payload_bytes: レスポンスペイロード（Protobuf シリアライズ済みバイト列）
    pub payload_bytes: Vec<u8>,
    // trace_id: レスポンスに含まれる trace_id（tracing 連携用）
    pub trace_id: String,
}

// RpcStatus は RPC の呼び出し結果ステータスを表す Library 独自型。
// gRPC status code を Library 独自語彙にマッピングする。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcStatus {
    // Ok: 正常完了
    Ok,
    // NotFound: リソースが見つからない（gRPC NOT_FOUND に対応）
    NotFound,
    // Unauthenticated: 認証エラー（gRPC UNAUTHENTICATED に対応）
    Unauthenticated,
    // PermissionDenied: 認可エラー（gRPC PERMISSION_DENIED に対応）
    PermissionDenied,
    // ResourceExhausted: quota / rate limit 超過（gRPC RESOURCE_EXHAUSTED に対応）
    ResourceExhausted,
    // Unavailable: サービス不到達（gRPC UNAVAILABLE に対応）
    Unavailable,
    // DeadlineExceeded: タイムアウト（gRPC DEADLINE_EXCEEDED に対応）
    DeadlineExceeded,
    // Internal: 内部エラー（gRPC INTERNAL に対応）
    Internal(String),
}

// RpcClient は RPC/Gateway の L3 抽象 trait（backend 向け）。
// tonic（gRPC）/ HTTP/2 等の OSS を実装で切り替えられる。
// auth context 伝播 / retry / tracing は ServicePolicy 経由で強制する。
#[async_trait]
pub trait RpcClient: Send + Sync {
    // call は RpcRequest を受け取り、RpcResponse を返す。
    // auth_ctx は認証コンテキストをリクエストヘッダーに注入するために使用する。
    // span_ctx は W3C traceparent ヘッダーとして注入する（None は root span を生成する）。
    // policy は retry / timeout / circuit breaker を適用するために使用する。
    async fn call(
        &self,
        request: RpcRequest,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<RpcResponse>;

    // health_check は呼び出し先サービスの health を確認する。
    // 実装は gRPC Health Checking Protocol に従う。
    async fn health_check(&self, service_name: &str) -> Result<bool>;

    // resolve_service は service_name から接続先エンドポイントを解決する。
    // DNS-SD / Kubernetes Service 等を実装で切り替えられる。
    async fn resolve_service(&self, service_name: &str) -> Result<String>;
}
