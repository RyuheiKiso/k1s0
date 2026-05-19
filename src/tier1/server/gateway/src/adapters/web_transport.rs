// web_transport.rs — spec 01 Bidi §adapter web_transport
// 全 5 conformance_class をサポートする WebTransport H/3 adapter。
// requires_fallback=true: WebTransport 非対応 UA は connect_bidi / sse_paired にフォールバック。
// 現環境では wtransport crate の C ライブラリ依存が解決できないため、
// HTTP/1.1 upgrade endpoint として動作し、非対応の場合は paired_post_sse へのフォールバック情報を返す。
// UA 判定でクライアントの WebTransport 対応能力を検出し、BidiSession handshake を駆動する。

// axum のレスポンス型・ルーター・ルーティング関数をインポートする
use axum::{
    // Router: axum のルーティング構造体
    Router,
    // http::StatusCode: HTTP ステータスコードを表す型
    http::StatusCode,
    // http::HeaderMap: HTTP リクエストヘッダーを受け取る型
    http::HeaderMap,
    // response::IntoResponse: handler の戻り値を HTTP Response に変換するトレイト
    response::{IntoResponse, Response},
    // routing: HTTP メソッドルーターを提供するモジュール
    routing::{get, post},
    // body::Body: HTTP レスポンスボディ型
    body::Body,
    // extract::State: axum State 依存性注入（EventBus を handler に注入する）
    extract::State,
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
};
// serde で JSON デシリアライズを行うためにインポートする
use serde::Deserialize;
// serde_json で JSON オブジェクトを構築するためにインポートする
use serde_json::json;
// std::sync::Arc: EventBus の shared state 共有に使用する
use std::sync::Arc;
// crate::event_bus::EventBus: セッションごとの broadcast channel 管理
use crate::event_bus::EventBus;
// 親モジュールの AdapterManifest 型をインポートする
use super::AdapterManifest;
// 同一 crate 内の BidiSession 型をインポートする（handshake state machine を使用する）
use crate::bidi::BidiSession;

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

// MIN_CHROMIUM_VERSION: WebTransport をサポートする Chromium の最小バージョン
const MIN_CHROMIUM_VERSION: u32 = 97;

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

    // detect_webtransport_capable は UA 文字列から WebTransport 対応能力を検出する。
    // true: WebTransport 対応（Chromium 97+ が確認できた場合）
    // false: 非対応または判定不能（fallback が必要）
    pub fn detect_webtransport_capable(user_agent: &str) -> bool {
        // "Chrome/" または "Chromium/" を UA 文字列から探す
        let chrome_prefix = if user_agent.contains("Chrome/") {
            Some("Chrome/")
        } else if user_agent.contains("Chromium/") {
            Some("Chromium/")
        } else {
            // Chrome / Chromium 以外は WebTransport 非対応とみなす
            None
        };

        // Chrome / Chromium バージョンを抽出して MIN_CHROMIUM_VERSION と比較する
        if let Some(prefix) = chrome_prefix {
            // "Chrome/" または "Chromium/" の後のバージョン文字列を取得する
            if let Some(version_str) = user_agent.split(prefix).nth(1) {
                // バージョン番号の major 部分（最初のドットまで）を取得する
                let major_str = version_str.split('.').next().unwrap_or("0");
                // major バージョンを u32 にパースする（パース失敗時は 0 とする）
                let major_version: u32 = major_str.parse().unwrap_or(0);
                // MIN_CHROMIUM_VERSION 以上なら WebTransport 対応とみなす
                return major_version >= MIN_CHROMIUM_VERSION;
            }
        }
        // 判定不能の場合は非対応とみなす（fallback を指示する）
        false
    }
}

// WebTransportConnectRequest は /connect エンドポイントへのリクエストを宣言する。
#[derive(Debug, Deserialize)]
pub struct WebTransportConnectRequest {
    // session_id: クライアントが生成した UUID v4
    pub session_id: Option<String>,
    // conformance_class: 要求する Bidi class（未指定時は v1_interactive）
    pub conformance_class: Option<String>,
}

// WebTransportState は web_transport handler に注入する共有状態を宣言する。
pub struct WebTransportState {
    // event_bus: DomainEvent を broadcast channel で配信する（将来的な WebTransport session に使う）
    pub event_bus: Arc<EventBus>,
}

// WebTransport の利用可否チェックエンドポイントのハンドラー。
// UA ヘッダーで WebTransport 対応を判定し、available フラグをレスポンスに含める。
// 非対応の場合は fallback adapter として paired_post_sse を推奨する。
pub async fn handle_webtransport_check(
    // headers: HTTP リクエストヘッダーを受け取る（User-Agent で UA 判定を行う）
    headers: HeaderMap,
) -> impl IntoResponse {
    // User-Agent ヘッダーから UA 文字列を取得する
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // UA 文字列から WebTransport 対応能力を検出する
    let wt_capable = WebTransportAdapter::detect_webtransport_capable(user_agent);

    // WebTransport 利用可否・フォールバック先・理由を JSON で返す
    let body = json!({
        // available: UA が WebTransport 対応かつ H/3 が構成済みの場合に true
        // NOTE: 現環境では HTTP/3 QUIC が未構成のため常に false を返す
        "available": false,
        // ua_capable: UA が WebTransport 対応かどうか（H/3 設定と独立した判定）
        "ua_capable": wt_capable,
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
// UA 判定で WebTransport 対応が確認できた場合は BidiSession handshake を駆動する。
// 非対応 UA の場合は 503 + X-WT-Fallback ヘッダーでフォールバック先を通知する。
pub async fn handle_webtransport_connect(
    // State(state): axum State 依存性注入で WebTransportState を受け取る
    State(state): State<Arc<WebTransportState>>,
    // headers: HTTP リクエストヘッダーを受け取る（User-Agent で UA 判定を行う）
    headers: HeaderMap,
    // body: JSON リクエストボディを受け取る（session_id / conformance_class を取得する）
    Json(body): Json<WebTransportConnectRequest>,
) -> impl IntoResponse {
    // User-Agent ヘッダーから UA 文字列を取得する
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // UA 文字列から WebTransport 対応能力を検出する
    let wt_capable = WebTransportAdapter::detect_webtransport_capable(user_agent);

    // WebTransport 非対応 UA の場合は paired_post_sse へのフォールバックを指示する
    if !wt_capable {
        // 非対応 UA をログに記録する
        tracing::info!(
            // user_agent フィールドを構造化ログに含める
            user_agent = %user_agent,
            "web_transport: UA not WebTransport capable, directing to paired_post_sse fallback"
        );
        // WebTransport が利用不可能であることを 503 で通知する
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("content-type", "application/json")
            // X-WT-Fallback ヘッダーでフォールバック先 adapter を通知する
            .header("x-wt-fallback", "paired_post_sse")
            // Retry-After は設定しない（UA 変更は自動再試行では解決しないため）
            .body(Body::from(
                json!({
                    // エラーコード: ua_not_capable
                    "code": "ua_not_capable",
                    // エラーメッセージ: WebTransport 非対応 UA
                    "message": "WebTransport requires Chromium 97+ (QUIC/HTTP3)",
                    // フォールバック先を明示する
                    "fallback": "paired_post_sse",
                    // フォールバック先のエンドポイントを明示する
                    "fallback_endpoints": {
                        "up": "/post-sse/bidi/up",
                        "down": "/post-sse/bidi/down",
                    },
                })
                .to_string(),
            ))
            .unwrap();
    }

    // session_id を取得する（未指定時は UUID v4 で生成する）
    let session_id = body.session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    // conformance_class を取得する（未指定時は v1_interactive をデフォルトとする）
    let conformance_class = body.conformance_class
        .unwrap_or_else(|| "v1_interactive".to_string());

    // WebTransport 対応 UA に対して BidiSession handshake を駆動する
    let mut session = BidiSession::new(
        session_id.clone(),
        "web_transport".to_string(),
        conformance_class.clone(),
    );

    // BidiSession の Hello 送信フェーズを実行する（TLA+ SendHello アクションに対応する）
    if let Err(e) = session.send_hello() {
        // SendHello 失敗は handshake violation を示す（NoDoubleHandshake 不変条件）
        tracing::warn!(error = %e, session_id = %session_id, "web_transport: send_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }

    // BidiSession の Hello 受信フェーズを実行する（サーバー→クライアント方向の Hello 受信）
    if let Err(e) = session.receive_hello() {
        // ReceiveHello 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "web_transport: receive_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }

    // BidiSession の Ack 送信フェーズを実行する（TLA+ SendAck アクションに対応する）
    if let Err(e) = session.send_ack() {
        // SendAck 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "web_transport: send_ack failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }

    // BidiSession を完了状態（Done）に遷移させる
    if let Err(e) = session.complete() {
        // complete 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "web_transport: complete failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }

    // WebTransport セッション確立成功をログに記録する
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %conformance_class,
        // handshake 完了フラグをログに記録する
        state = "Done",
        "web_transport: BidiSession handshake completed (simulated, QUIC not configured)"
    );

    // EventBus の subscribe 受け口を確保する（将来の WebTransport stream に使う）
    // NOTE: 現環境では QUIC が未構成のため、ここで subscribe を記録するのみ
    let _subscriber_hint = state.event_bus.subscribe(&session_id);

    // WebTransport セッション確立レスポンスを返す
    // NOTE: 実際の WebTransport では HTTP/3 CONNECT を使うが、現環境では JSON で模擬する
    let response_body = json!({
        // session_id: 確立したセッションの識別子を返す
        "session_id": session_id,
        // state: BidiSession の状態（Done）
        "state": "Done",
        // adapter: 使用した adapter ID
        "adapter": "web_transport",
        // conformance_class: 確立した Bidi class
        "conformance_class": conformance_class,
        // note: 現環境での制約を通知する
        "note": "QUIC/HTTP3 not configured; session established over HTTP/1.1 (simulated)",
        // fallback_available: paired_post_sse への fallback が利用可能であることを示す
        "fallback_available": true,
        // fallback: fallback adapter の名前
        "fallback": "paired_post_sse",
    });
    // 200 OK でセッション確立レスポンスを返す
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .header("x-session-id", session_id)
        .body(Body::from(response_body.to_string()))
        .unwrap()
}

// web_transport adapter の axum Router を返す。
// EventBus を State として注入し、/check と /connect を登録する。
pub fn router(event_bus: Arc<EventBus>) -> Router {
    // WebTransportState を生成して axum State に注入する
    let state = Arc::new(WebTransportState { event_bus });
    // GET /webtransport/check と POST /webtransport/connect を登録する
    Router::new()
        // WebTransport 利用可否チェックエンドポイントを GET として登録する
        .route("/check", get(handle_webtransport_check))
        // WebTransport upgrade/connect エンドポイントを POST として登録する
        .route("/connect", post(handle_webtransport_connect))
        // WebTransportState を axum State として注入する
        .with_state(state)
}
