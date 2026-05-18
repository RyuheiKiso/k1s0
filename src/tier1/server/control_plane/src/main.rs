// k1s0 tier1 control_plane: CRD watcher + flagd publisher のエントリポイント
// Tier1Service CRD の変更を kube::runtime::watcher で監視し、
// テナント別 feature flag を OpenFeature flagd に配布する。
// SIGTERM / SIGINT でのグレースフルシャットダウンに対応する。

// CRD watcher モジュール（kube 0.95 runtime::watcher API）
mod crd_watcher;
// flagd publisher モジュール（ConfigMap 経由の OpenFeature flagd feature flag 配布）
mod flagd_publisher;

// axum: ヘルスチェック用 HTTP サーバー
use axum::{Json, Router, routing::get};
// serde: JSON シリアライズ
use serde::Serialize;
// tokio: 非同期ランタイム + shutdown signal
use tokio::sync::broadcast;
// tracing: 構造化ロギング
use tracing::{error, info, warn};
// tracing-subscriber: サブスクライバー
use tracing_subscriber::EnvFilter;
// 標準ライブラリ
use std::env;
use std::sync::Arc;

// crd_watcher モジュールから型を import する
use crd_watcher::{CrdWatchEvent, CrdWatcher};
// flagd_publisher モジュールから FlagdPublisher を import する
use flagd_publisher::FlagdPublisher;

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

// AppState はグローバルアプリケーション状態を保持する。
// axum handler に注入して動的なステータス確認に使用する。
#[derive(Clone)]
struct AppState {
    // kubernetes_connected: Kubernetes API に接続済みかどうか
    kubernetes_connected: Arc<std::sync::atomic::AtomicBool>,
}

// health_handler はヘルスチェックエンドポイントのハンドラー。
async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<ControlPlaneStatus> {
    // flagd への接続設定を環境変数から確認する
    let flagd_configured = env::var("FLAGD_HOST").is_ok();
    // Kubernetes 接続状態を atomic bool から取得する
    let kube_connected = state
        .kubernetes_connected
        .load(std::sync::atomic::Ordering::SeqCst);
    // ControlPlaneStatus を構築して返す
    Json(ControlPlaneStatus {
        status: "healthy".to_string(),
        service: "k1s0-tier1-cp".to_string(),
        crd_watcher_active: kube_connected,
        flagd_connected: flagd_configured,
        kubernetes_connected: kube_connected,
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
        .with_target(true)
        .init();

    // リスニングアドレスを環境変数から取得する
    let addr = env::var("CP_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8083".to_string());
    info!(
        addr = %addr,
        "k1s0-tier1-cp starting"
    );

    // shutdown signal の broadcast channel を構築する
    // SIGTERM または Ctrl+C で shutdown_tx.send(()) が呼ばれる
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // Kubernetes 接続状態を管理する atomic bool
    let kubernetes_connected = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let kube_connected_clone = kubernetes_connected.clone();

    // CrdWatcher を構築して watch を開始する
    let watcher = CrdWatcher::new();
    let shutdown_rx_for_watcher = shutdown_tx.subscribe();

    // CrdWatcher の watch を開始して broadcast Receiver を取得する
    let mut event_rx = match watcher.watch(shutdown_rx_for_watcher).await {
        Ok(rx) => {
            // watch 開始成功時は kubernetes_connected を true に設定する
            kubernetes_connected.store(true, std::sync::atomic::Ordering::SeqCst);
            info!("CrdWatcher: watch started successfully");
            rx
        }
        Err(e) => {
            // watch 開始失敗時は警告を出して degraded mode で続行する
            warn!(error = %e, "CrdWatcher: failed to start watch, operating in degraded mode");
            // 受信者がいないダミー broadcast channel を作成する
            let (_, rx) = broadcast::channel::<CrdWatchEvent>(1);
            rx
        }
    };

    // CRD イベントを処理するバックグラウンドタスクを起動する
    let kube_connected_for_event = kube_connected_clone.clone();
    tokio::spawn(async move {
        // FlagdPublisher を構築する（Kubernetes API への接続が必要）
        // kubernetes_connected が false（degraded mode）の場合は Publisher は非活性となる
        let maybe_publisher: Option<FlagdPublisher> = {
            // kube::Client を再取得して FlagdPublisher を初期化する
            match kube::Client::try_default().await {
                Ok(client) => {
                    // Kubernetes API 接続成功時は FlagdPublisher を構築する
                    Some(FlagdPublisher::new(client))
                }
                Err(_) => {
                    // Kubernetes API 接続失敗時は FlagdPublisher を無効化する（degraded mode）
                    warn!("FlagdPublisher: kube::Client 接続失敗 — degraded mode で動作する");
                    None
                }
            }
        };
        // CrdWatchEvent を受信して処理するループ
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    // Kubernetes 接続が確認できたため true に設定する
                    kube_connected_for_event.store(true, std::sync::atomic::Ordering::SeqCst);
                    // イベントの種別に応じて flagd publisher に転送する
                    match &event {
                        CrdWatchEvent::Added { namespace, name, spec } => {
                            // 新しい Tier1Service が作成された
                            info!(
                                namespace = %namespace,
                                name = %name,
                                conformance_class = %spec.conformance_class,
                                adapter = %spec.adapter,
                                "CrdWatcher: Tier1Service Added"
                            );
                            // flagd_publisher で feature flag を ConfigMap に配布する
                            if let Some(ref publisher) = maybe_publisher {
                                // ConfigMap 経由で flagd に feature flag を publish する
                                if let Err(e) = publisher
                                    .publish(namespace, name, &spec.conformance_class, &spec.adapter)
                                    .await
                                {
                                    // 配布失敗は警告のみ（CRD watcher ループは継続する）
                                    warn!(error = %e, "FlagdPublisher: Added publish 失敗");
                                }
                            }
                        }
                        CrdWatchEvent::Modified { namespace, name, spec } => {
                            // 既存の Tier1Service が変更された
                            info!(
                                namespace = %namespace,
                                name = %name,
                                conformance_class = %spec.conformance_class,
                                adapter = %spec.adapter,
                                "CrdWatcher: Tier1Service Modified"
                            );
                            // flagd_publisher で feature flag を更新する（publish で冪等 apply）
                            if let Some(ref publisher) = maybe_publisher {
                                // 既存 ConfigMap を server-side apply で上書きする（冪等性）
                                if let Err(e) = publisher
                                    .publish(namespace, name, &spec.conformance_class, &spec.adapter)
                                    .await
                                {
                                    // 更新失敗は警告のみ
                                    warn!(error = %e, "FlagdPublisher: Modified publish 失敗");
                                }
                            }
                        }
                        CrdWatchEvent::Deleted { namespace, name } => {
                            // Tier1Service が削除された
                            info!(
                                namespace = %namespace,
                                name = %name,
                                "CrdWatcher: Tier1Service Deleted"
                            );
                            // flagd_publisher で feature flag の ConfigMap を削除する
                            if let Some(ref publisher) = maybe_publisher {
                                // 対応する ConfigMap を削除する（存在しない場合は冪等に成功する）
                                if let Err(e) = publisher.delete(namespace, name).await {
                                    // 削除失敗は警告のみ
                                    warn!(error = %e, "FlagdPublisher: Deleted delete 失敗");
                                }
                            }
                        }
                    }
                }
                // broadcast channel が終了した（watcher が shutdown された）
                Err(broadcast::error::RecvError::Closed) => {
                    info!("CrdWatcher event channel closed, stopping event handler");
                    break;
                }
                // broadcast channel のバッファが溢れた（イベントを取りこぼした）
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    warn!(count = count, "CrdWatcher event channel lagged, {} events missed", count);
                }
            }
        }
    });

    // OS シグナル (SIGTERM / SIGINT) でのシャットダウンを処理するタスクを起動する
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        // Ctrl+C を待機する（SIGTERM は tokio の signal で処理する）
        match tokio::signal::ctrl_c().await {
            Ok(_) => {
                info!("k1s0-tier1-cp: SIGINT received, initiating graceful shutdown");
            }
            Err(e) => {
                error!(error = %e, "k1s0-tier1-cp: signal handler error");
            }
        }
        // shutdown signal を broadcast する
        let _ = shutdown_tx_clone.send(());
    });

    // axum アプリケーション状態を構築する
    let app_state = AppState {
        kubernetes_connected: kubernetes_connected.clone(),
    };

    // ヘルスチェック HTTP サーバーを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイント（Kubernetes liveness / readiness probe）
        .route("/health", get(health_handler))
        .with_state(app_state);

    // TCP リスナーを起動する
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "k1s0-tier1-cp HTTP server listening");

    // shutdown signal を受け取ったらサーバーを停止する
    let shutdown_signal = async move {
        // shutdown_rx を subscribe して shutdown signal を待つ
        let mut rx = shutdown_tx.subscribe();
        let _ = rx.recv().await;
        info!("k1s0-tier1-cp: HTTP server shutdown initiated");
    };

    // axum サーバーを graceful shutdown 対応で起動する
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("k1s0-tier1-cp: shutdown complete");
    Ok(())
}
