// web_transport.rs — spec 01 Bidi §adapter web_transport
// 全 5 conformance_class をサポートする WebTransport H/3 adapter。
// requires_fallback=true: WebTransport 非対応 UA は connect_bidi / sse_paired にフォールバック。
// 現環境では wtransport crate の C ライブラリ依存が解決できないため、
// HTTP/1.1 upgrade endpoint として動作し、非対応の場合は paired_post_sse へのフォールバック情報を返す。

// axum のレスポンス型・ルーター・ルーティング関数をインポートする
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    body::Body,
};
// serde_json で JSON オブジェクトを構築するためにインポートする
use serde_json::json;
// 親モジュールの AdapterManifest 型をインポートする
use super::AdapterManifest;

// MANIFEST は web_transport adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "web_transport",
    // web_transport は全 5 conformance_class をサポートする（v1 opt-in）
    supports: &[
        "v1_interactive",    // 双方向対話 WebTransport bidi stream
        "v1_alert",          // サーバー主導警報 WebTransport server stream
        "v1_event_feed",     // Domain Event 配信 WebTransport server stream
        "v1_live_snapshot",  // 最新値表示 WebTransport server stream
        "v1_bulk_upload",    // 大量データ投入 WebTransport client stream
    ],
    // WebTransport は Chromium 97+ のみ対応。非対応 UA は fallback が必須
    requires_fallback: true,
    // HTTP/3 QUIC が必要（UDP 443 を開放する必要がある）
    constraints: "http3=required, quic=required, ua_min=chromium97",
};

// WebTransportAdapter は WebTransport H/3 adapter。
pub struct WebTransportAdapter;

impl WebTransportAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }
}

// WebTransport の利用可否チェックエンドポイントのハンドラー。
// 現環境では HTTP/3 QUIC が未構成のため available=false を返し、
// fallback adapter として paired_post_sse を推奨する。
pub async fn handle_webtransport_check() -> impl IntoResponse {
    // WebTransport 利用可否・フォールバック先・理由を JSON で返す
    let body = json!({
        // 現環境では HTTP/3 QUIC が構成されていないため利用不可を示す
        "available": false,
        // フォールバック先として paired_post_sse を指定する（HTTP/1.1 SSE pair）
        "fallback": "paired_post_sse",
        // 利用不可の理由: h3_quic_not_configured（Envoy/Nginx で QUIC を有効化する必要がある）
        "reason": "h3_quic_not_configured",
        // 対応 UA の最低要件（Chromium 97 以上が必要）
        "ua_min": "chromium97",
        // 現在の adapter_id を返す
        "adapter_id": "web_transport",
    });
    // 200 OK で利用可否 JSON を返す（クライアントは available フィールドで判断する）
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

// WebTransport upgrade/connect エンドポイントのハンドラー。
// WebTransport が利用不可のため 503 Service Unavailable を返し、
// X-WT-Fallback ヘッダーでフォールバック先を通知する。
pub async fn handle_webtransport_connect() -> impl IntoResponse {
    // WebTransport が利用不可能であることを 503 で通知する
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("content-type", "application/json")
        // X-WT-Fallback ヘッダーでフォールバック先 adapter を通知する
        .header("x-wt-fallback", "paired_post_sse")
        // Retry-After は設定しない（QUIC 開放は手動設定のため自動再試行は無意味）
        .body(Body::from(
            json!({
                // エラーコード: service_unavailable
                "code": "service_unavailable",
                // エラーメッセージ: WebTransport 未構成
                "message": "WebTransport over HTTP/3 is not available: h3_quic_not_configured",
                // フォールバック先を明示する
                "fallback": "paired_post_sse",
            })
            .to_string(),
        ))
        .unwrap()
}

// web_transport adapter の axum Router を返す。
// main.rs で .nest("/webtransport", web_transport::router()) として使用する。
pub fn router() -> Router {
    // GET /webtransport/check と POST /webtransport/connect を登録する
    Router::new()
        // WebTransport 利用可否チェックエンドポイントを GET として登録する
        .route("/check", get(handle_webtransport_check))
        // WebTransport upgrade/connect エンドポイントを POST として登録する
        .route("/connect", post(handle_webtransport_connect))
}
