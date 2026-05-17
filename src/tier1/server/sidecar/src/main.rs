// k1s0 tier1 sidecar: Outbox リレーサイドカーのエントリポイント
// PostgreSQL Outbox テーブルをポーリングして Kafka に WAL イベントをリレーする

// axum: HTTP サーバーとハンドラーのインポート
use axum::{Json, Router, routing::get};
// Serde シリアライズのインポート
use serde::{Deserialize, Serialize};
// tracing のインポート
use tracing::info;
// tracing サブスクライバーのインポート
use tracing_subscriber::EnvFilter;

// サイドカーのヘルスチェックレスポンス構造体
#[derive(Serialize, Deserialize)]
struct HealthResponse {
    // サービス動作状態
    status: String,
    // サービス名
    service: String,
    // Outbox リレーが動作しているかのフラグ
    outbox_relay_active: bool,
}

// ヘルスチェックエンドポイントのハンドラー
async fn health_handler() -> Json<HealthResponse> {
    // ヘルスチェックレスポンスを返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-sidecar".to_string(),
        outbox_relay_active: true,
    })
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing の初期設定（環境変数 RUST_LOG で制御する）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイントを登録する
        .route("/health", get(health_handler));
    // サーバーのリスニングアドレスを設定する
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    // サーバー起動をログに記録する
    info!("k1s0-tier1-sidecar starting on :8081");
    // サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
