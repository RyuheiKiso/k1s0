// k1s0 tier1 backend_for_library: ライブラリ向けバックエンドのエントリポイント
// AuthContext 型を公開 API の必須型とし、生 token を公開シグネチャに露出しない

// axum: HTTP サーバーとハンドラーのインポート
use axum::{Json, Router, routing::{get, post}};
// Serde シリアライズのインポート
use serde::{Deserialize, Serialize};
// tracing のインポート
use tracing::info;
// tracing サブスクライバーのインポート
use tracing_subscriber::EnvFilter;
// UUID のインポート
use uuid::Uuid;

// AuthContext: 生 token を露出しない opaque 型（公開 API の必須型）
// 生の access_token / refresh_token をフィールドに持たない設計にする
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    // 相関 ID（リクエストトレーシングに使用する）
    pub correlation_id: String,
    // テナント ID（RLS GUC 注入に使用する）
    pub tenant_id: String,
    // サービスアカウント名（権限チェックに使用する）
    pub service_account: String,
    // トークンが有効期限内かどうかのフラグ（生 token は保持しない）
    pub is_valid: bool,
}

// トークン検証リクエストの構造体（生 token は入力としてのみ受け取る）
#[derive(Deserialize)]
struct VerifyTokenRequest {
    // 検証対象のトークン（opaque string として扱い内部で破棄する）
    token: String,
    // テナント ID
    tenant_id: String,
}

// ヘルスチェックレスポンスの構造体
#[derive(Serialize)]
struct HealthResponse {
    // サービス動作状態
    status: String,
    // サービス名
    service: String,
}

// トークン検証エンドポイントのハンドラー
// 生 token を受け取り AuthContext を返す（生 token は応答に含めない）
async fn verify_token_handler(
    Json(req): Json<VerifyTokenRequest>,
) -> Json<AuthContext> {
    // kind 環境では全トークンを有効として扱う（テスト用）
    // 本番では Keycloak / OpenBao Transit と連携する
    let is_valid = !req.token.is_empty();
    // 相関 ID を生成する
    let correlation_id = Uuid::new_v4().to_string();
    // トークン検証をログに記録する（生 token はログに出さない）
    info!(
        tenant_id = %req.tenant_id,
        correlation_id = %correlation_id,
        is_valid,
        "Token verification completed"
    );
    // AuthContext を返す（生 token は応答に含めない）
    Json(AuthContext {
        correlation_id,
        tenant_id: req.tenant_id,
        service_account: "k1s0-svc".to_string(),
        is_valid,
    })
}

// ヘルスチェックエンドポイントのハンドラー
async fn health_handler() -> Json<HealthResponse> {
    // ヘルスチェックレスポンスを返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-bfl".to_string(),
    })
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing の初期設定
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイントを登録する
        .route("/health", get(health_handler))
        // トークン検証エンドポイントを登録する
        .route("/auth/verify", post(verify_token_handler));
    // サーバーのリスニングアドレスを設定する
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await?;
    // サーバー起動をログに記録する
    info!("k1s0-tier1-bfl starting on :8082");
    // サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
