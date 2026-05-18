// rpc.rs — k1s0 tier1 Library frontend: RPC/Gateway L3 thin wrapper（frontend 向け）
// frontend 向けの RPC/Gateway thin wrapper を定義する。
// backend::rpc の型を再利用し、frontend 固有の動作（CORS / Credentials / AbortSignal）を追加する。
// frontend RPC は HTTP/1.1 + JSON / gRPC-Web のみをサポートする（TCP gRPC は backend 専用）。
// auth context の自動注入と tracing ヘッダーの付与は backend 版と同じ。
// 公開 API に OSS 型（reqwest::Client / tonic::Channel 等）を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: リクエスト / レスポンス型のシリアライズに使用する
use serde::{Deserialize, Serialize};

// backend::rpc の型を frontend 向けに re-export する
// （frontend も RpcRequest / RpcResponse / RpcStatus を使用するため）
pub use crate::backend::rpc::{RpcRequest, RpcResponse, RpcStatus};
// core::auth の AuthContext を re-export する
pub use crate::core::auth::AuthContext;
// core::policy の ServicePolicy を re-export する
pub use crate::core::policy::ServicePolicy;
// core::observability の SpanContext を re-export する
pub use crate::core::observability::SpanContext;

// FrontendRpcOptions は frontend RPC 呼び出しのオプションを表す Library 独自型。
// backend 版の ServicePolicy に加えて frontend 固有の設定を追加する。
#[derive(Debug, Clone)]
pub struct FrontendRpcOptions {
    // policy: retry / timeout / circuit breaker ポリシー（ServicePolicy と同一）
    pub policy: ServicePolicy,
    // credentials_mode: CORS の credentials モード（"omit" / "same-origin" / "include"）
    pub credentials_mode: CredentialsMode,
    // abort_after_ms: リクエストを中断するまでの待機時間（ミリ秒; 0 は無制限）
    pub abort_after_ms: u64,
}

// CredentialsMode は CORS の credentials モードを表す Library 独自型。
// OSS の reqwest::redirect::Policy 等には依存しない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialsMode {
    // Omit: Cookie / Authorization header を送らない（public API 向け）
    Omit,
    // SameOrigin: 同一オリジンのみ credentials を送る（デフォルト）
    SameOrigin,
    // Include: クロスオリジンでも credentials を送る（認証済み API 向け）
    Include,
}

// FrontendRpcOptions のデフォルト値
impl Default for FrontendRpcOptions {
    fn default() -> Self {
        // frontend 向け安全なデフォルト設定
        Self {
            // policy: デフォルト ServicePolicy（retry + timeout + circuit breaker）
            policy: ServicePolicy::default(),
            // credentials_mode: SameOrigin（デフォルト; クロスオリジン credentials は明示指定が必要）
            credentials_mode: CredentialsMode::SameOrigin,
            // abort_after_ms: 0（無制限; policy の timeout が実質的な上限）
            abort_after_ms: 0,
        }
    }
}

// FrontendRpcResponse は frontend RPC 呼び出しの結果を表す Library 独自型。
// backend 版の RpcResponse に HTTP status code を追加する。
#[derive(Debug, Clone)]
pub struct FrontendRpcResponse {
    // base: backend 版の RpcResponse（payload_bytes / status_code / trace_id）
    pub base: RpcResponse,
    // http_status: HTTP ステータスコード（gRPC-Web の場合は gRPC status を HTTP に変換した値）
    pub http_status: u16,
    // latency_ms: クライアント計測のレイテンシ（ミリ秒）
    pub latency_ms: u64,
}

// FrontendRpcClient は frontend 向けの RPC/Gateway L3 抽象 trait。
// HTTP/JSON / gRPC-Web を実装で切り替えられる。
// auth context 伝播 / retry / tracing は FrontendRpcOptions 経由で強制する。
#[async_trait]
pub trait FrontendRpcClient: Send + Sync {
    // call は RpcRequest を受け取り、FrontendRpcResponse を返す。
    // auth_ctx は Authorization ヘッダーに自動注入する（Bearer トークン形式）。
    // span_ctx は W3C traceparent ヘッダーとして注入する。
    // options は retry / timeout / credentials / abort を制御する。
    async fn call(
        &self,
        request: RpcRequest,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        options: &FrontendRpcOptions,
    ) -> Result<FrontendRpcResponse>;

    // call_unauthenticated は認証なしの RPC 呼び出しを行う（公開 API 向け）。
    // auth_ctx を受け取らない（Authorization ヘッダーを付けない）。
    async fn call_unauthenticated(
        &self,
        request: RpcRequest,
        span_ctx: Option<&SpanContext>,
        options: &FrontendRpcOptions,
    ) -> Result<FrontendRpcResponse>;

    // base_url は呼び出し先の base URL を返す（デバッグ / ログ用）
    fn base_url(&self) -> &str;
}
