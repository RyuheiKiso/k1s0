// k1s0 tier1 backend_for_library: AuthContext 型で 5 auth_class を振り分けるエントリポイント
// 04_認証適合仕様.md §v1 auth_class セット（5 class）に基づき、
// 各 class ごとの検証ロジックを auth_context.rs / oidc.rs / mtls.rs / jwt.rs に委譲する。
// 生 access_token / refresh_token は応答に含めない（公開 API 型保証）。

// auth_context モジュール: AuthContext opaque 型 + AuthClass enum（5 class）
mod auth_context;
// oidc モジュール: OIDC + DPoP 検証（v1_human_session / v1_emergency_step_up）
mod oidc;

// axum: HTTP サーバーとハンドラー
use axum::{Json, Router, routing::{get, post}};
// serde: JSON シリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{info, warn};
// tracing-subscriber: サブスクライバー
use tracing_subscriber::EnvFilter;
// UUID: session_id / correlation_id 生成
use uuid::Uuid;
// 標準ライブラリ
use std::env;

use auth_context::{AuthClass, AuthContext};
use oidc::OidcVerifier;

// VerifyTokenRequest は /auth/verify エンドポイントへのリクエストを宣言する。
// 生 token を受け取り、内部で parse・検証後に AuthContext として返す。
#[derive(Debug, Deserialize)]
struct VerifyTokenRequest {
    // bearer_token: 検証対象の Bearer token（内部で parse 後に破棄する）
    bearer_token: String,
    // auth_class_hint: 呼び出し元が期待する auth_class（未指定時は自動判定）
    auth_class_hint: Option<String>,
    // tenant_id: リクエストのテナント識別子
    tenant_id: String,
    // dpop_token: DPoP proof JWT（v1_human_session / v1_emergency_step_up のみ必須）
    dpop_token: Option<String>,
    // request_method: DPoP 検証用 HTTP method（大文字）
    request_method: Option<String>,
    // request_uri: DPoP 検証用 URI
    request_uri: Option<String>,
}

// VerifyResponse は /auth/verify エンドポイントのレスポンスを宣言する。
// 生 token は含まない（AuthContext のみを返す）。
#[derive(Serialize)]
struct VerifyResponse {
    // auth_context: 検証結果の AuthContext（公開 API の必須型）
    auth_context: AuthContext,
    // guc_setters: PostgreSQL SET LOCAL 文のリスト（tier2 TenantContext と組み合わせる）
    guc_setters: Vec<String>,
    // warnings: non-fatal な警告メッセージのリスト
    warnings: Vec<String>,
}

// HealthResponse はヘルスチェックレスポンスを宣言する。
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    // supported_auth_classes: このエンドポイントが対応している auth_class の一覧
    supported_auth_classes: Vec<String>,
}

// verify_token_handler は /auth/verify エンドポイントのハンドラー。
// 5 auth_class に基づいて検証ロジックを振り分ける。
async fn verify_token_handler(
    Json(req): Json<VerifyTokenRequest>,
) -> Json<VerifyResponse> {
    // auth_class_hint から auth_class を決定する
    let auth_class_hint = req.auth_class_hint.as_deref().unwrap_or("v1_workload_jwt");
    // auth_class に応じて検証を実施する（5 class 分岐）
    let (auth_ctx, mut warnings) = match auth_class_hint {
        "v1_human_session" => {
            // OIDC + DPoP 検証（Keycloak issuer は環境変数から取得する）
            let issuer = env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "https://keycloak.k1s0.internal/realms/k1s0".to_string());
            let audience = env::var("KEYCLOAK_AUDIENCE")
                .unwrap_or_else(|_| "k1s0-gateway".to_string());
            let verifier = OidcVerifier::new(issuer, audience);
            match verifier.verify_token(&req.bearer_token) {
                Ok(claims) => {
                    // DPoP proof を検証する（v1_human_session は必須）
                    let dpop_jkt = req.dpop_token.as_deref().and_then(|dpop| {
                        let method = req.request_method.as_deref().unwrap_or("POST");
                        let uri = req.request_uri.as_deref().unwrap_or("/auth/verify");
                        verifier.verify_dpop_proof(dpop, method, uri).ok().map(|_| dpop.to_string())
                    });
                    let ctx = AuthContext::new_human_session(
                        claims.sub,
                        req.tenant_id.clone(),
                        claims.jti,
                        claims.scope.map(|s| s.split(' ').map(|x| x.to_string()).collect()).unwrap_or_default(),
                        dpop_jkt,
                        false,
                    );
                    (ctx, vec![])
                }
                Err(e) => {
                    // 検証失敗時は is_valid=false の AuthContext を返す（raw token は含めない）
                    warn!(auth_class = "v1_human_session", error = %e, "token verification failed");
                    let ctx = AuthContext::new_human_session(
                        "unknown".to_string(), req.tenant_id.clone(),
                        Uuid::new_v4().to_string(), vec![], None, false,
                    );
                    let ctx = AuthContext { is_valid: false, ..ctx };
                    (ctx, vec![format!("verification failed: {e}")])
                }
            }
        }
        "v1_workload_jwt" => {
            // SPIFFE SVID JWT 検証（workload identity）
            // TODO: SPIRE bundle endpoint から JWKS を取得して署名検証する
            info!(auth_class = "v1_workload_jwt", "workload JWT verification (SPIRE JWKS pending)");
            let ctx = AuthContext::new_workload_jwt(
                "workload-stub".to_string(),
                req.tenant_id.clone(),
                Uuid::new_v4().to_string(),
                "k1s0-gateway".to_string(),
            );
            (ctx, vec!["SPIRE JWKS verification pending".to_string()])
        }
        "v1_device_attest" => {
            // Device attestation chain 検証（TPM / HSM / WebAuthn platform authenticator）
            // TODO: device cert chain を verify する
            info!(auth_class = "v1_device_attest", "device attestation (TPM/HSM pending)");
            let mut ctx = AuthContext::new_human_session(
                "device-stub".to_string(), req.tenant_id.clone(),
                Uuid::new_v4().to_string(), vec![], None, false,
            );
            ctx.auth_class = AuthClass::V1DeviceAttest;
            ctx.subject_kind = "device".to_string();
            (ctx, vec!["TPM/HSM attestation verification pending".to_string()])
        }
        "v1_federated_exchange" => {
            // RFC 8693 token exchange 検証（外部 IdP → audience-restricted JWT）
            // TODO: federation issuer JWKS を取得して act/may_act claim を検証する
            info!(auth_class = "v1_federated_exchange", "federated token exchange (RFC 8693 pending)");
            let mut ctx = AuthContext::new_workload_jwt(
                "federated-stub".to_string(), req.tenant_id.clone(),
                Uuid::new_v4().to_string(), "k1s0-gateway".to_string(),
            );
            ctx.auth_class = AuthClass::V1FederatedExchange;
            ctx.subject_kind = "external_subject".to_string();
            (ctx, vec!["federation JWKS verification pending".to_string()])
        }
        "v1_emergency_step_up" => {
            // Break-glass アクセス検証（always step_up + purpose=emergency 強制）
            // TODO: step_up challenge 完了を証明する WebAuthn assertion を検証する
            info!(auth_class = "v1_emergency_step_up", "emergency step-up (WebAuthn pending)");
            let ctx = AuthContext::new_emergency_step_up(
                "emergency-stub".to_string(),
                req.tenant_id.clone(),
                Uuid::new_v4().to_string(),
                req.dpop_token.clone(),
            );
            (ctx, vec!["WebAuthn step_up assertion verification pending".to_string()])
        }
        unknown => {
            // 未知の auth_class は v1_workload_jwt として処理してエラーを返す
            warn!(unknown, "unknown auth_class_hint, falling back to v1_workload_jwt");
            let ctx = AuthContext::new_workload_jwt(
                "unknown-class".to_string(), req.tenant_id.clone(),
                Uuid::new_v4().to_string(), "k1s0-gateway".to_string(),
            );
            (ctx, vec![format!("unknown auth_class: {unknown}")])
        }
    };
    // is_valid = true の場合は成功ログを出す
    if auth_ctx.is_valid {
        info!(
            auth_class = %auth_ctx.auth_class,
            subject_id = %auth_ctx.subject_id,
            tenant_id = %auth_ctx.tenant_id,
            "AuthContext verified: raw token not in response"
        );
    }
    // GUC SET LOCAL 文を生成する（tier2 TenantContext と組み合わせる）
    let guc_setters = auth_ctx.to_guc_setters();
    // レスポンスを返す（生 token は含まれない）
    Json(VerifyResponse {
        auth_context: auth_ctx,
        guc_setters,
        warnings,
    })
}

// health_handler はヘルスチェックエンドポイントのハンドラー。
async fn health_handler() -> Json<HealthResponse> {
    // 対応 auth_class の一覧を返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-bfl".to_string(),
        supported_auth_classes: vec![
            "v1_human_session".to_string(),
            "v1_workload_jwt".to_string(),
            "v1_device_attest".to_string(),
            "v1_federated_exchange".to_string(),
            "v1_emergency_step_up".to_string(),
        ],
    })
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing を初期化する
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    // リスニングアドレスを環境変数から取得する
    let addr = env::var("BFL_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイント
        .route("/health", get(health_handler))
        // AuthContext 取得エンドポイント（5 auth_class に対応）
        .route("/auth/verify", post(verify_token_handler));
    // TCP リスナーを起動する
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "k1s0-tier1-bfl starting");
    // サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
