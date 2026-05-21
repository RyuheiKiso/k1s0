// k1s0 tier1 sidecar: Outbox → Kafka relay サービスのエントリポイント
// tier2 atomic_triple_write の outbox_message を PostgreSQL から読み出し Kafka に relay する。
// WAL-based CDC（pgoutput logical replication）も対応する。

// Outbox relay モジュール（PostgreSQL SELECT FOR UPDATE SKIP LOCKED → Kafka producer）
mod outbox_relay;
// envoy_jwt_authn_config モジュール: spec 04 §5 層 D Envoy jwt_authn filter 設定 generator
mod envoy_jwt_authn_config;

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

use outbox_relay::{OutboxRelay, OutboxRelayConfig};

// SidecarStatus はサイドカーの稼働状態を宣言する。
#[derive(Serialize)]
struct SidecarStatus {
    // status: サービス動作状態
    status: String,
    // service: サービス名
    service: String,
    // outbox_relay_active: Outbox relay が稼働中かどうか
    outbox_relay_active: bool,
    // kafka_connected: Kafka broker に接続済みかどうか
    kafka_connected: bool,
    // database_connected: PostgreSQL に接続済みかどうか
    database_connected: bool,
}

// health_handler はヘルスチェックエンドポイントのハンドラー。
async fn health_handler() -> Json<SidecarStatus> {
    // 各コンポーネントの接続状態を環境変数から確認する
    let kafka_configured = env::var("KAFKA_BROKERS").is_ok();
    let database_configured = env::var("DATABASE_URL").is_ok();
    Json(SidecarStatus {
        status: "healthy".to_string(),
        service: "k1s0-tier1-sidecar".to_string(),
        // Outbox relay は DATABASE_URL + KAFKA_BROKERS が設定されていれば active
        outbox_relay_active: kafka_configured && database_configured,
        kafka_connected: kafka_configured,
        database_connected: database_configured,
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
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://k1s0:k1s0@localhost:5432/k1s0".to_string());
    // kafka_brokers はカンマ区切りで複数 broker を指定できる（rskafka は Vec<String> を受け付ける）
    let kafka_brokers_str = env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    // カンマ区切りを Vec<String> に変換する
    let kafka_brokers: Vec<String> = kafka_brokers_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let poll_interval_ms: u64 = env::var("OUTBOX_POLL_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    let max_retry_count: i32 = env::var("OUTBOX_MAX_RETRY_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    // batch_size: 1 回のポーリングで取得する最大メッセージ数
    let batch_size: i64 = env::var("OUTBOX_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    // Outbox relay を初期化する
    let relay = OutboxRelay::new(OutboxRelayConfig {
        database_url: database_url.clone(),
        kafka_brokers: kafka_brokers.clone(),
        poll_interval_ms,
        max_retry_count,
        batch_size,
    });
    // リスニングアドレスを環境変数から取得する
    let addr = env::var("SIDECAR_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    info!(
        addr = %addr,
        database = %database_url,
        kafka = ?kafka_brokers,
        "k1s0-tier1-sidecar starting"
    );
    // ヘルスチェック HTTP サーバーと Outbox relay を並行して起動する
    let app = Router::new()
        .route("/health", get(health_handler));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    // Outbox relay を別タスクで起動する
    tokio::spawn(async move {
        // Outbox relay ループを起動する（エラー時はログに記録して継続する）
        if let Err(e) = relay.run().await {
            tracing::error!(error = %e, "OutboxRelay terminated with error");
        }
    });
    // ヘルスチェック HTTP サーバーを起動する
    axum::serve(listener, app).await?;
    Ok(())
}
