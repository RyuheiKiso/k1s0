// paired_post_sse.rs — spec 01 Bidi §adapter paired_post_sse
// POST↔SSE pair adapter: 半二重 emulation（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）。
// v1_bulk_upload は半二重 emulation では担えないため not_applicable。
// 12_UA_aware_adapter.md の K1s0UaAwareAdapter と連携して UA 判定を行う。
// 13_dotnet8_connect_inhouse.md の .NET Framework 対応に使用する。

// axum Router・POST ハンドラー・SSE ハンドラーで使用する型を import する
use axum::{
    // Router: axum のルーティング構造体
    Router,
    // routing: HTTP メソッドルーターを提供するモジュール
    routing::{get, post},
    // extract::Query: クエリパラメータを抽出する Extractor
    extract::Query,
    // response::sse: SSE（Server-Sent Events）型を提供するモジュール
    response::sse::{Event, KeepAlive, Sse},
    // response::IntoResponse: 各 handler の戻り値を HTTP Response に変換するトレイト
    response::IntoResponse,
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
    // http::HeaderMap: HTTP ヘッダーマップ型
    http::{HeaderMap, StatusCode},
};
// async_stream::stream! マクロで非同期 Stream を生成するために import する
use async_stream::stream;
// serde::Deserialize: クエリパラメータ型を JSON / QueryString からデシリアライズするために使用する
use serde::Deserialize;
// std::convert::Infallible: SSE Stream の Item エラー型に使用する（stream が失敗しないことを示す）
use std::convert::Infallible;
// AdapterManifest: adapter の capability 自己宣言型
use super::AdapterManifest;

// MANIFEST は paired_post_sse adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "paired_post_sse",
    // paired_post_sse は半二重 emulation のため client→server bulk は不向き
    supports: &[
        "v1_interactive",    // 双方向対話 POST↔SSE pair（UA aware adapter 経由）
        "v1_alert",          // サーバー主導警報 POST↔SSE
        "v1_event_feed",     // Domain Event 配信 POST↔SSE
        "v1_live_snapshot",  // 最新値表示 POST↔SSE
    ],
    // HTTP/1.1 SSE と POST の pair なので fallback 不要（全ブラウザ対応）
    requires_fallback: false,
    // SSE endpoint + POST endpoint の pair が必要、connect-inhouse compat あり
    constraints: "legacy_compat=true, connect_inhouse=supported, half_duplex=true",
};

// PairedPostSseAdapter は POST↔SSE pair adapter。
pub struct PairedPostSseAdapter;

impl PairedPostSseAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        // MANIFEST から adapter_id を参照して返す
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        // MANIFEST.supports スライスに conformance_class が含まれるか確認する
        MANIFEST.supports.contains(&conformance_class)
    }
}

// DownQuery は GET /bidi/down エンドポイントのクエリパラメータを宣言する。
// request_id で up stream と SSE stream を対応付ける。
#[derive(Deserialize)]
pub struct DownQuery {
    // request_id: POST /bidi/up と SSE /bidi/down を紐付ける識別子
    pub request_id: String,
    // session_id: BidiSession の識別子（上流 POST と対応させる）
    pub session_id: String,
    // conformance_class: 要求する Bidi class（省略可能）
    pub conformance_class: Option<String>,
    // last_event_id: SSE resume 時の前回最終 event ID（Last-Event-ID ヘッダー相当）
    pub last_event_id: Option<String>,
}

// handle_up は POST /bidi/up の handler。
// client → server 方向のメッセージを受け取り、request_id で SSE down stream と対応付ける。
// wall clock を使わない（TTL 計算には HLC を使う — 本 handler は TTL 計算なし）。
pub async fn handle_up(
    // headers: HTTP リクエストヘッダーを受け取る（X-Request-Id 取得に使用する）
    headers: HeaderMap,
    // body: JSON リクエストボディを受け取る（session_id / request_id / payload を取得する）
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    // X-Request-Id ヘッダーから request_id を取得する（なければ body の request_id を使う）
    let request_id_from_header = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // body から session_id を取得する（なければ空文字を使う）
    let session_id = body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // body の request_id を取得する（ヘッダー優先でフォールバックとして使う）
    let request_id = request_id_from_header
        .or_else(|| {
            body.get("request_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    // body から payload を取得する（なければ null を使う）
    let payload = body
        .get("payload")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // 受け取ったメッセージを構造化ログに記録する（tracing を使う）
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // request_id フィールドを構造化ログに含める
        request_id = %request_id,
        // payload のサイズをログに残す
        payload_type = %payload.to_string().len(),
        "paired_post_sse up message received"
    );

    // 受信確認を JSON で返す（200 OK + { "received": true, "session_id": ... }）
    let response_body = serde_json::json!({
        // received: クライアントへの受信確認フラグ
        "received": true,
        // session_id: リクエストの session_id をエコーバックする
        "session_id": session_id,
        // request_id: 対応する SSE down stream の識別子
        "request_id": request_id,
    });

    // 200 OK ステータスコードと JSON ボディを返す
    (StatusCode::OK, Json(response_body))
}

// handle_down は GET /bidi/down の handler。
// server → client 方向の SSE stream を返す。
// KeepAlive で接続を維持し、session_started event と heartbeat を送信する。
pub async fn handle_down(
    // params: クエリパラメータを DownQuery 型として受け取る
    Query(params): Query<DownQuery>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    // session_id をキャプチャして SSE stream 内で使用する
    let session_id = params.session_id.clone();
    // request_id をキャプチャして SSE stream 内で使用する
    let request_id = params.request_id.clone();
    // conformance_class をキャプチャして SSE stream 内で参照する
    let conformance_class = params
        .conformance_class
        .clone()
        .unwrap_or_else(|| "v1_event_feed".to_string());
    // last_event_id がある場合は resume_token として使用する
    let last_event_id = params.last_event_id.clone().unwrap_or_default();

    // tracing で SSE down stream の開始をログに記録する
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // request_id フィールドを構造化ログに含める
        request_id = %request_id,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %conformance_class,
        "paired_post_sse down SSE stream opened"
    );

    // async_stream::stream! マクロで非同期 Stream を生成する
    let event_stream = stream! {
        // session_started イベントを最初に送信する（クライアントに接続確立を通知する）
        let session_started_data = serde_json::json!({
            // session_id: セッション識別子をクライアントに通知する
            "session_id": session_id,
            // request_id: 対応する POST up stream の識別子をクライアントに通知する
            "request_id": request_id,
            // conformance_class: 確立した Bidi class をクライアントに通知する
            "conformance_class": conformance_class,
            // resume_from: resume_token（last_event_id）をエコーバックする
            "resume_from": last_event_id,
        });
        // session_started イベントを yield する
        yield Ok(
            // event 名を "session_started" に設定する
            Event::default()
                .event("session_started")
                // session_id を SSE id フィールドに設定する（resume 時の Last-Event-ID に使用する）
                .id(&session_id)
                // セッション開始データを JSON 文字列で data フィールドに設定する
                .data(session_started_data.to_string())
        );

        // heartbeat イベントを一定間隔で送信する（接続を維持するため）
        // NOTE: 実際の broadcast channel からの event receive は将来実装する
        //       現在は heartbeat のみを送信して接続を維持する
        let heartbeat_data = serde_json::json!({
            // type: heartbeat であることを示す
            "type": "heartbeat",
            // session_id: セッション識別子をクライアントに通知する
            "session_id": session_id,
        });
        // heartbeat イベントを yield する
        yield Ok(
            // event 名を "heartbeat" に設定する
            Event::default()
                .event("heartbeat")
                // heartbeat データを JSON 文字列で data フィールドに設定する
                .data(heartbeat_data.to_string())
        );
    };

    // Sse::new で stream を wrap し、KeepAlive で接続を維持する
    Sse::new(event_stream)
        // KeepAlive::default() で接続を維持する（15s ごとに comment を送信する）
        .keep_alive(KeepAlive::default())
}

// router は paired_post_sse adapter の axum Router を構築して返す。
// /bidi/up に POST ハンドラー、/bidi/down に GET（SSE）ハンドラーを登録する。
pub fn router() -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // POST /bidi/up: client → server メッセージ受付エンドポイント
        .route("/bidi/up", post(handle_up))
        // GET /bidi/down: server → client SSE streaming エンドポイント
        .route("/bidi/down", get(handle_down))
}
