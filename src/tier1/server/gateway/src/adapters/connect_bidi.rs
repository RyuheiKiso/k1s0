// connect_bidi.rs — spec 01 Bidi §adapter connect_bidi
// 全 5 conformance_class をサポートする Connect-RPC bidi adapter。
// fetch full-duplex streams を使用し、UA subclass 別に adapter を選択する。
// 13_dotnet8_connect_inhouse.md の Connect-RPC inhouse 実装と連携する。

// axum の JSON extractor・レスポンス型・ルーター・ルーティング関数をインポートする
use axum::{
    Json,
    Router,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    body::Body,
};
// serde で JSON デシリアライズを行うためにインポートする
use serde::Deserialize;
// serde_json で JSON オブジェクトを構築するためにインポートする
use serde_json::json;
// tracing で構造化ログを記録するためにインポートする
use tracing::info;
// 親モジュールの AdapterManifest 型をインポートする
use super::AdapterManifest;
// 同一 crate 内の BidiSession 型をインポートする（handshake state machine を使用する）
use crate::bidi::BidiSession;

// MANIFEST は connect_bidi adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "connect_bidi",
    // connect_bidi は全 5 conformance_class をサポートする（ua_subclass 別 map で対応）
    supports: &[
        "v1_interactive",    // 双方向対話 Connect bidi streaming
        "v1_alert",          // サーバー主導警報 Connect server streaming
        "v1_event_feed",     // Domain Event 配信 Connect server streaming
        "v1_live_snapshot",  // 最新値表示 Connect server streaming
        "v1_bulk_upload",    // 大量データ投入 Connect client streaming
    ],
    // connect_bidi は HTTP/2 TLS が前提だが UA によっては paired_post_sse にフォールバック
    requires_fallback: false,
    // fetch full-duplex は Chrome 119+ / Firefox 113+ / Safari 18+ が必要
    constraints: "fetch_full_duplex=required, ua_min=chrome119",
};

// ConnectBidiAdapter は Connect-RPC bidi adapter。
pub struct ConnectBidiAdapter;

impl ConnectBidiAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }
}

// Connect-RPC streaming リクエストのボディ構造体。
// Connect プロトコルでは JSON または proto3 のいずれかで送信される。
#[derive(Debug, Deserialize)]
pub struct ConnectStreamRequest {
    // セッション ID: クライアントが生成した UUID v4
    pub session_id: Option<String>,
    // 対象の conformance_class（未指定時は v1_interactive をデフォルトとする）
    pub conformance_class: Option<String>,
}

// Connect-RPC streaming ハンドラー。
// POST /connect/bidi/stream を受け付け、BidiSession handshake を実行して結果を返す。
pub async fn handle_connect_stream(
    headers: HeaderMap,
    Json(body): Json<ConnectStreamRequest>,
) -> impl IntoResponse {
    // Content-Type ヘッダーを取得して Connect-RPC 対応の型であることを確認する
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // application/connect+proto または application/json または application/connect+json を許可する
    let is_valid_ct = content_type.starts_with("application/connect+proto")
        || content_type.starts_with("application/connect+json")
        || content_type.starts_with("application/json");
    // 不正な Content-Type の場合は 415 Unsupported Media Type を返す
    if !is_valid_ct {
        // Connect-RPC プロトコルで認められない Content-Type を拒否する
        return Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header("content-type", "application/connect+json")
            .body(Body::from(
                json!({
                    "code": "invalid_argument",
                    "message": "Content-Type must be application/connect+proto or application/connect+json"
                })
                .to_string(),
            ))
            .unwrap();
    }
    // session_id を取得する（未指定時は "unknown" を使用する）
    let session_id = body.session_id.unwrap_or_else(|| "unknown".to_string());
    // conformance_class を取得する（未指定時は v1_interactive をデフォルトとする）
    let conformance_class = body
        .conformance_class
        .unwrap_or_else(|| "v1_interactive".to_string());
    // connect_bidi adapter で BidiSession を生成して handshake を開始する
    let mut session = BidiSession::new(
        session_id.clone(),
        "connect_bidi".to_string(),
        conformance_class.clone(),
    );
    // BidiSession の Hello 送信フェーズを実行する（TLA+ SendHello アクションに対応する）
    if let Err(e) = session.send_hello() {
        // SendHello 失敗は handshake violation を示す（NoDoubleHandshake 不変条件）
        tracing::warn!(error = %e, session_id = %session_id, "connect_bidi: send_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/connect+json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }
    // BidiSession の Hello 受信フェーズを実行する（サーバー→クライアント方向の Hello 受信）
    if let Err(e) = session.receive_hello() {
        // ReceiveHello 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "connect_bidi: receive_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/connect+json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }
    // BidiSession の Ack 送信フェーズを実行する（TLA+ SendAck アクションに対応する）
    if let Err(e) = session.send_ack() {
        // SendAck 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "connect_bidi: send_ack failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/connect+json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }
    // BidiSession を完了状態（Done）に遷移させる
    if let Err(e) = session.complete() {
        // complete 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "connect_bidi: complete failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/connect+json")
            .body(Body::from(
                json!({"code": "failed_precondition", "message": e.to_string()}).to_string(),
            ))
            .unwrap();
    }
    // セッション完了をトレースに記録する
    info!(
        session_id = %session_id,
        conformance_class = %conformance_class,
        state = "Done",
        invariant_ok = session.check_invariant(),
        "connect_bidi: handshake completed"
    );
    // BidiSession の状態を Connect-RPC 形式の JSON レスポンスとして返す
    let response_body = json!({
        "session_id": session_id,
        "state": "Done",
        "adapter": "connect_bidi",
        "conformance_class": conformance_class,
        "send_count": session.send_count,
        "recv_count": session.recv_count,
        "invariant_ok": session.check_invariant(),
    });
    // Content-Type: application/connect+json で正常レスポンスを返す
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/connect+json")
        .header("x-session-id", session_id)
        .body(Body::from(response_body.to_string()))
        .unwrap()
}

// connect_bidi adapter の axum Router を返す。
// main.rs で .nest("/connect", connect_bidi::router()) として使用する。
pub fn router() -> Router {
    // POST /connect/bidi/stream に Connect-RPC streaming ハンドラーを登録する
    Router::new()
        // Connect-RPC streaming エンドポイントを POST として登録する
        .route("/bidi/stream", post(handle_connect_stream))
}
