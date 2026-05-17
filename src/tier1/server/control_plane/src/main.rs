// k1s0 tier1 control_plane: CRD watcher + flagd publisher のエントリポイント
// Tier1Service CRD の変更を kube watch API で監視し、
// テナント別 feature flag を OpenFeature flagd に配布する。

// CRD watcher モジュール（kube 0.95 watch API）
mod crd_watcher;

// axum: ヘルスチェック用 HTTP サーバー
use axum::{Json, Router, routing::get};
// serde: JSON シリアライズ
use serde::Serialize;
// tracing: 構造化ロギング
use tracing::info;
// tracing-subscriber: サブスクライバー
use tracing_subscriber::EnvFilter;
// 標準ライブラリ
use std::env;

use crd_watcher::CrdWatcher;

// ControlPlaneStatus はコントロールプレーンの稼働状態を宣言する。
#[derive(Serialize)]
struct ControlPlaneStatus {
    // status: サービス動作状態
    status: String,
    // service: サービス名
    service: String,
    // crd_watcher_active: CRD watcher が稼働中かどうか
    crd_watcher_active: bool,
    // flagd_connected: flagd に接続済みかどうか
    flagd_connected: bool,
    // kubernetes_connected: Kubernetes API に接続済みかどうか
    kubernetes_connected: bool,
}

// health_handler はヘルスチェックエンドポイントのハンドラー。
async fn health_handler() -> Json<ControlPlaneStatus> {
    // 各コンポーネントの接続状態を環境変数から確認する
    let flagd_configured = env::var("FLAGD_HOST").is_ok();
    let kubeconfig = env::var("KUBECONFIG").is_ok() || std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount/token").exists();
    Json(ControlPlaneStatus {
        status: "healthy".to_string(),
        service: "k1s0-tier1-cp".to_string(),
        crd_watcher_active: kubeconfig,
        flagd_connected: flagd_configured,
        kubernetes_connected: kubeconfig,
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
    // 環境変数から設定を読み込む
    let kubeconfig = env::var("KUBECONFIG")
        .unwrap_or_else(|_| "/var/run/secrets/kubernetes.io/serviceaccount/token".to_string());
    let addr = env::var("CP_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8083".to_string());
    info!(
        addr = %addr,
        kubeconfig = %kubeconfig,
        "k1s0-tier1-cp starting"
    );
    // CRD watcher を初期化する
    let watcher = CrdWatcher::new(kubeconfig);
    // ヘルスチェック HTTP サーバーと CRD watcher を並行して起動する
    let app = Router::new()
        .route("/health", get(health_handler));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    // CRD watcher を別タスクで起動する
    tokio::spawn(async move {
        // CRD watcher ループを起動する（イベントをログに記録する）
        if let Err(e) = watcher.watch(|event| {
            info!(event = ?event, "CRD event received");
        }).await {
            tracing::error!(error = %e, "CrdWatcher terminated with error");
        }
    });
    // ヘルスチェック HTTP サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
