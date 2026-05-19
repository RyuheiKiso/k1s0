// lib.rs — k1s0-tier1-gateway ライブラリエントリポイント
// integration test（tests/ 配下）が build_router() を利用できるように pub に再エクスポートする。
// binary エントリポイントは main.rs が引き続き担当する。

// Bidi state machine モジュール（TLA+ NoDoubleHandshake 不変条件）
pub mod bidi;
// conformance テストランナーモジュール（5 class × 8 adapter の Bidi conformance 検証）
pub mod conformance;
// KeyHandle opaque 型モジュール（5 key_class + zeroize RAII）
pub mod key_handle;
// capability negotiation モジュール（chosen_transport 選択アルゴリズム）
pub mod capability_negotiation;
// 8 adapter モジュール群
pub mod adapters;
// event_bus モジュール（セッションごとの broadcast channel 管理 — long_poll adapter が使用）
pub mod event_bus;
// scenario_runner モジュール（scenarios.yaml から Bidi シナリオを読み込んで実行する）
pub mod scenario_runner;

// axum: HTTP ルーター（use される識別子のみインポートする）
use axum::{Json, Router, routing::get, extract::Query, http::StatusCode, response::IntoResponse};
// base64::Engine: STANDARD.encode を使うために trait を scope に入れる必要がある
use base64::Engine as _;
// serde: JSON シリアライズ
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギング
use tracing::{info, instrument, warn};
// 標準ライブラリ
use std::env;

// capability_negotiation の公開型をインポートする
use capability_negotiation::{NegotiationRequest, NegotiationResult, negotiate};
// key_handle の公開型をインポートする
use key_handle::{KeyClass, KeyHandle};
// event_bus の EventBus をインポートする（long_poll adapter に注入する）
use event_bus::EventBus;

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
    // 全 40 conformance cell を実行する（5 class × 8 adapter）
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

// KeyHandleDemoResponse は /kek/demo エンドポイントのレスポンス型
#[derive(Serialize)]
struct KeyHandleDemoResponse {
    // handle: KeyHandle（key_bytes は #[serde(skip)] により JSON 出力に含まれない）
    handle: KeyHandle,
    // sign_verified: OpenBao Transit sign が成功したことを示すフラグ
    sign_verified: bool,
    // demo_payload_b64: 署名対象として使用した fixed payload の Base64 表現
    demo_payload_b64: String,
}

// KeyHandle デモエンドポイント（spec 05 鍵管理: 生 key bytes を返さない API 型保証）
// OpenBao Transit 経由で実際の sign を実行し、key bytes が公開 API に含まれないことを実証する。
#[instrument]
async fn key_handle_demo_handler() -> impl IntoResponse {
    // デモ署名対象ペイロード: 固定バイト列（"k1s0-kek-demo-proof" の UTF-8 バイト）
    let demo_payload = b"k1s0-kek-demo-proof";
    // handle_id: このデモリクエスト固有の UUID v4 を生成する
    let handle_id = uuid::Uuid::new_v4().to_string();
    // handle: OpenBao Transit の v1_data_dek キーへの参照として KeyHandle を構築する
    let handle = KeyHandle::from_remote_handle(KeyClass::V1DataDek, handle_id.clone());
    // client: 環境変数（OPENBAO_ADDR / OPENBAO_TOKEN / OPENBAO_TRANSIT_MOUNT）から構築する
    let client = key_handle::OpenBaoTransitClient::from_env();
    // OpenBao Transit に sign を要求して実際の署名バイト列を取得する
    match client.sign(&handle, demo_payload).await {
        Ok(signature) => {
            // 署名成功: signature が空でないことを確認する（OpenBao の空返却はエラー）
            let sign_verified = !signature.is_empty();
            // ログに署名成功を記録する（bytes 値は出力しない）
            info!(
                handle_id = %handle_id,
                key_class = %handle.key_class_str(),
                is_valid = handle.is_valid,
                signature_len = signature.len(),
                "KeyHandle demo: OpenBao sign succeeded, raw bytes not in response",
            );
            // レスポンスを構築して返す（key_bytes は KeyHandle の #[serde(skip)] により除外される）
            (
                StatusCode::OK,
                Json(KeyHandleDemoResponse {
                    // handle を JSON に含める（key_bytes は除外される）
                    handle,
                    // 署名成功フラグを設定する
                    sign_verified,
                    // デモ payload を Base64 エンコードして返す
                    demo_payload_b64: base64::engine::general_purpose::STANDARD.encode(demo_payload),
                }),
            )
                .into_response()
        }
        Err(e) => {
            // OpenBao 接続失敗: 503 Service Unavailable を返す（OpenBao 未起動 or token 無効）
            warn!(
                handle_id = %handle_id,
                error = %e,
                "KeyHandle demo: OpenBao sign failed (check OPENBAO_ADDR / OPENBAO_TOKEN)",
            );
            // 503 と エラーメッセージを返す
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "OpenBao Transit unavailable",
                    "detail": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

// build_router は gateway の axum Router を構築して返す。
// pub: integration test（tests/ 配下）と main.rs の両方から参照する。
// EventBus を作成して long_poll adapter に注入する。
pub fn build_router() -> Router {
    // EventBus を生成する（gateway 全体で 1 インスタンス共有）
    let event_bus = EventBus::new();
    // 全エンドポイントを登録する
    Router::new()
        // ヘルスチェックエンドポイント（Kubernetes liveness / readiness probe）
        .route("/health", get(health_handler))
        // conformance テストエンドポイント（40 cell の Bidi conformance 検証）
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
        // adapter 6: long_poll — fetch long-poll（EventBus を注入して broadcast channel を配線する）
        .nest("/long-poll", adapters::long_poll::router(event_bus))
        // adapter 7: messaging_bridge — Kafka idempotent producer（/kafka/...）
        .nest("/kafka", adapters::messaging_bridge::router())
        // adapter 8: webhook — HMAC-SHA256 signed webhook イベント受信（/webhook/...）
        .nest("/webhook", adapters::webhook::router())
}
