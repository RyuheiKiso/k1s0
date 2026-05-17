// k1s0 tier1 gateway: HTTP/2 (h2c) サーバーエントリポイント
// spec 01 Bidi: 5 conformance_class × 8 adapter の transport 基盤
// spec 03 観測: OTel tracing + W3C traceparent 伝搬
// spec 05 鍵管理: KeyHandle opaque 型による生 key bytes 隠蔽
//
// TLS / ALPN h2 強制は Envoy proxy が担当する（mTLS サービスメッシュ構成）。
// gateway は h2c（HTTP/2 cleartext）で listen し、Envoy の後段として動作する。
// Envoy の ALPN ネゴシエーションで h2 のみを許可する設定は src/_crosscutting/01_http2_enforcement/ を参照。

// Bidi state machine モジュール（TLA+ NoDoubleHandshake 不変条件）
mod bidi;
// conformance テストランナーモジュール（5 class × 8 adapter の Bidi conformance 検証）
mod conformance;
// KeyHandle opaque 型モジュール（5 key_class + zeroize RAII）
mod key_handle;
// capability negotiation モジュール（chosen_transport 選択アルゴリズム）
mod capability_negotiation;
// 8 adapter モジュール群
mod adapters;

// axum: HTTP ルーター（http2 feature で h2c 対応）
use axum::{Json, Router, routing::get, routing::post, extract::Query, response::IntoResponse};
// serde: JSON シリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{info, instrument};
// tracing-subscriber: サブスクライバー
use tracing_subscriber::EnvFilter;
// 標準ライブラリ
use std::env;

use capability_negotiation::{NegotiationRequest, NegotiationResult, negotiate};
use key_handle::{KeyClass, KeyHandle};

// ヘルスチェックレスポンスの構造体
#[derive(Serialize, Deserialize)]
struct HealthResponse {
    // サービスの動作状態
    status: String,
    // サービス名
    service: String,
    // HTTP/2 h2c モードで動作していることを示す（ALPN は Envoy proxy で強制）
    http2_mode: String,
    // OTel OTLP Collector への接続フラグ
    otel_otlp_configured: bool,
}

// capability negotiation エンドポイントのクエリパラメータ
#[derive(Debug, Deserialize)]
struct NegotiateQuery {
    // 要求する conformance_class
    conformance_class: Option<String>,
    // 優先 adapter をカンマ区切りで指定する
    preferred_adapters: Option<String>,
    // UA ヒント
    ua_hint: Option<String>,
}

// health check エンドポイントのハンドラー
#[instrument]
async fn health_handler() -> Json<HealthResponse> {
    // h2c モードで動作中（Envoy proxy が ALPN h2 を強制する）
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-gateway".to_string(),
        // gateway は h2c で動作し、Envoy が TLS + ALPN h2 を担当する
        http2_mode: "h2c-behind-envoy".to_string(),
        // OTel OTLP は環境変数 OTEL_EXPORTER_OTLP_ENDPOINT で設定する
        otel_otlp_configured: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok(),
    })
}

// conformance テストエンドポイントのハンドラー（5 class × 8 adapter）
#[instrument]
async fn conformance_handler() -> Json<conformance::ConformanceReport> {
    // 全 29 applicable conformance cell を実行する（11 not_applicable は除外）
    let report = conformance::run_all_conformance_tests("kind-k1s0-target");
    // conformance テスト結果をトレースに記録する
    info!(
        all_passed = report.all_passed,
        total_cells = report.cells.len(),
        "conformance test completed"
    );
    Json(report)
}

// capability negotiation エンドポイントのハンドラー
#[instrument]
async fn negotiate_handler(Query(params): Query<NegotiateQuery>) -> Json<NegotiationResult> {
    // クエリパラメータから NegotiationRequest を構築する
    let preferred_adapters = params.preferred_adapters
        .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
        .unwrap_or_default();
    let req = NegotiationRequest {
        conformance_class: params.conformance_class.unwrap_or_else(|| "v1_interactive".to_string()),
        preferred_adapters,
        ua_hint: params.ua_hint,
    };
    // 最適な adapter を選択する
    let result = negotiate(&req);
    // 選択結果をトレースに記録する
    info!(
        chosen = %result.chosen_adapter_name,
        fallback = result.fallback_used,
        "capability negotiation completed"
    );
    Json(result)
}

// KeyHandle デモエンドポイント（spec 05 鍵管理: 生 key bytes を返さない API 型保証）
#[instrument]
async fn key_handle_demo_handler() -> Json<KeyHandle> {
    // 5 key_class のうち v1_data_dek クラスの stub KeyHandle を生成する
    let handle = KeyHandle::create_stub(
        KeyClass::V1DataDek,
        uuid::Uuid::new_v4().to_string(),
    );
    // KeyHandle を JSON で返す（key_bytes は Serialize から除外されている）
    info!(
        key_class = %handle.key_class_str(),
        is_valid = handle.is_valid,
        "KeyHandle demo: raw bytes not in response"
    );
    Json(handle)
}

// axum router を構築する
fn build_router() -> Router {
    // 全エンドポイントを登録する
    Router::new()
        // ヘルスチェックエンドポイント（Kubernetes liveness / readiness probe）
        .route("/health", get(health_handler))
        // conformance テストエンドポイント（29 applicable cell の Bidi conformance 検証）
        .route("/conformance/run", get(conformance_handler))
        // capability negotiation エンドポイント（chosen_transport 選択）
        .route("/negotiate", get(negotiate_handler))
        // KeyHandle デモエンドポイント（spec 05 鍵管理の API 型保証デモ）
        .route("/kek/demo", get(key_handle_demo_handler))
        // adapter 1: grpc_native — gRPC over HTTP/2（/grpc/...）
        .nest("/grpc", adapters::grpc_native::router())
        // adapter 2: connect_bidi — Connect-RPC bidi（/connect/...）
        .nest("/connect", adapters::connect_bidi::router())
        // adapter 3: web_transport — WebTransport H/3 check + fallback（/webtransport/...）
        .nest("/webtransport", adapters::web_transport::router())
        // adapter 4: paired_post_sse — POST↔SSE pair（/post-sse/...）
        .nest("/post-sse", adapters::paired_post_sse::router())
        // adapter 5: sse_paired — EventSource SSE（/sse-stream/...）
        .nest("/sse-stream", adapters::sse_paired::router())
        // adapter 6: long_poll — fetch long-poll（/long-poll/...）
        .nest("/long-poll", adapters::long_poll::router())
        // adapter 7: messaging_bridge — Kafka idempotent producer（/kafka/...）
        .nest("/kafka", adapters::messaging_bridge::router())
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing を初期化する（RUST_LOG 環境変数で log level を制御する）
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .init();
    // リスニングアドレスを環境変数から取得する（デフォルト: 0.0.0.0:8080）
    let addr = env::var("GATEWAY_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    // TCP リスナーを起動する（h2c モード: Envoy が TLS + ALPN 処理を担当する）
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(
        addr = %addr,
        mode = "h2c-behind-envoy",
        "k1s0-tier1-gateway starting"
    );
    // axum ルーターを構築して serve する（axum は http2 feature で h2c 自動ネゴシエーション）
    axum::serve(listener, build_router()).await?;
    Ok(())
}
