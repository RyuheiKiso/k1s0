// k1s0 tier1 control_plane: flagd 連携コントロールプレーンのエントリポイント
// テナント設定・フィーチャーフラグを flagd に配布するスタブ実装

// axum: HTTP サーバーとハンドラーのインポート
use axum::{Json, Router, routing::get};
// Serde シリアライズのインポート
use serde::{Deserialize, Serialize};
// tracing のインポート
use tracing::info;
// tracing サブスクライバーのインポート
use tracing_subscriber::EnvFilter;

// テナント設定レスポンスの構造体
#[derive(Serialize, Deserialize)]
struct TenantConfig {
    // テナント ID
    tenant_id: String,
    // テナントの quota class（テナント容量管理に使用する）
    quota_class: String,
    // テナントの clock integrity class
    clock_integrity_class: String,
    // テナントのフィーチャーフラグ状態
    features_enabled: Vec<String>,
}

// ヘルスチェックレスポンスの構造体
#[derive(Serialize)]
struct HealthResponse {
    // サービス動作状態
    status: String,
    // サービス名
    service: String,
}

// テナント設定配布エンドポイントのハンドラー
async fn tenant_config_handler() -> Json<Vec<TenantConfig>> {
    // kind 環境での代表的なテナント設定を返す
    Json(vec![
        // テナント A の設定
        TenantConfig {
            tenant_id: "tenant_a".to_string(),
            quota_class: "standard".to_string(),
            clock_integrity_class: "v1_intra_rack_ptp".to_string(),
            features_enabled: vec!["bidi_streaming".to_string(), "slo_monitoring".to_string()],
        },
        // テナント B の設定
        TenantConfig {
            tenant_id: "tenant_b".to_string(),
            quota_class: "premium".to_string(),
            clock_integrity_class: "v1_dc_chrony_stratum1".to_string(),
            features_enabled: vec![
                "bidi_streaming".to_string(),
                "slo_monitoring".to_string(),
                "pii_dedicated_cluster".to_string(),
            ],
        },
    ])
}

// ヘルスチェックエンドポイントのハンドラー
async fn health_handler() -> Json<HealthResponse> {
    // ヘルスチェックレスポンスを返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-cp".to_string(),
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
        // テナント設定配布エンドポイントを登録する
        .route("/tenants/config", get(tenant_config_handler));
    // サーバーのリスニングアドレスを設定する
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await?;
    // サーバー起動をログに記録する
    info!("k1s0-tier1-cp starting on :8083");
    // サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
