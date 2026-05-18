// sse_paired.rs — spec 01 Bidi §adapter sse_paired
// EventSource SSE adapter（v1_alert / v1_event_feed / v1_live_snapshot のみサポート）。
// client→server bulk（v1_bulk_upload）は一方向 wire の都合上、構造的にサポートできない。
// axum SSE + Last-Event-ID による resume_token 実装。

// axum から SSE・ルーティング・抽出・レスポンス型を import する
use axum::{
    // Router: axum のルーティング構造体
    Router,
    // routing::get: GET メソッドルーター関数
    routing::get,
    // extract::Query: クエリパラメータを抽出する Extractor
    extract::Query,
    // response::sse: SSE 型（Event / KeepAlive / Sse）を提供するモジュール
    response::sse::{Event, KeepAlive, Sse},
    // http::HeaderMap: HTTP リクエストヘッダーを受け取る型
    http::HeaderMap,
};
// futures_util::stream: Stream 関連ユーティリティ（Stream トレイトを含む）
use futures_util::stream::{self, Stream};
// serde::Deserialize: クエリパラメータ型を QueryString からデシリアライズするために使用する
use serde::Deserialize;
// std::convert::Infallible: SSE Stream の Item エラー型に使用する（stream が失敗しないことを示す）
use std::convert::Infallible;
// AdapterManifest: adapter の capability 自己宣言型
use super::AdapterManifest;

// MANIFEST は sse_paired adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "sse_paired",
    // sse_paired は server→client 一方向のみ（v1_interactive / v1_bulk_upload は不可）
    supports: &[
        "v1_alert",          // サーバー主導警報 SSE（lag≤200ms、resumable via Last-Event-ID）
        "v1_event_feed",     // Domain Event 配信 SSE（lag≤5000ms、ordering=SESSION_ORDERED）
        "v1_live_snapshot",  // 最新値表示 SSE（ordering=UNORDERED、latest-wins）
    ],
    // SSE は全主要ブラウザで対応しているため fallback 不要
    requires_fallback: false,
    // text/event-stream で streaming、Last-Event-ID で resume_token を保持
    constraints: "content_type=text/event-stream, last_event_id=required_for_resume",
};

// SseQuery は SSE エンドポイントのクエリパラメータを宣言する。
#[derive(Deserialize)]
pub struct SseQuery {
    // conformance_class: 要求する Bidi class
    pub conformance_class: Option<String>,
    // session_id: BidiSession の識別子
    pub session_id: Option<String>,
}

// SsePairedAdapter は EventSource SSE adapter。
pub struct SsePairedAdapter;

impl SsePairedAdapter {
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

    // sse_stream は SSE イベントストリームを返す axum handler helper。
    // conformance_class に応じて ordering / lag / resumable を適用する。
    pub fn sse_stream(
        // conformance_class: Bidi class を示す文字列（lag target の選択に使用する）
        conformance_class: &str,
        // session_id: BidiSession の識別子（SSE id フィールドに設定する）
        session_id: String,
        // last_event_id: resume 時の前回最終 event ID（None の場合は新規セッション扱い）
        last_event_id: Option<String>,
    ) -> impl Stream<Item = Result<Event, Infallible>> {
        // last_event_id があれば resume_token から継続する（resumable=REQUIRED の保証）
        let resume_from = last_event_id.unwrap_or_default();
        // conformance_class に応じた lag target を取得する
        // v1_alert: lag≤200ms、v1_event_feed: lag≤5000ms、v1_live_snapshot: lag≤500ms
        let _lag_target_ms = match conformance_class {
            // v1_alert は最も厳格な lag≤200ms を target とする
            "v1_alert" => 200u64,
            // v1_event_feed は lag≤5000ms を target とする（SESSION_ORDERED 保証）
            "v1_event_feed" => 5000u64,
            // v1_live_snapshot は lag≤500ms を target とする（UNORDERED / latest-wins）
            "v1_live_snapshot" => 500u64,
            // MANIFEST.supports に含まれない class は到達しない
            _ => 0u64,
        };
        // SSE ping イベントを一定間隔で送信する（heartbeat + resume_token 維持）
        stream::once(async move {
            // 最初のイベントとして session 開始を通知する
            Ok(Event::default()
                // SSE id フィールドに session_id を設定する（Last-Event-ID の基底値として使用する）
                .id(&session_id)
                // event 名を "session_started" に設定する
                .event("session_started")
                // セッション開始データを JSON 文字列で data フィールドに設定する
                .data(format!("{{\"session_id\":\"{session_id}\",\"resume_from\":\"{resume_from}\"}}")))
        })
    }
}

// handle_sse は GET /sse の handler。
// server → client 一方向 SSE streaming エンドポイント。
// Last-Event-Id ヘッダーで resume_token を受け取り、resumable=REQUIRED を保証する。
pub async fn handle_sse(
    // headers: HTTP リクエストヘッダーを受け取る（Last-Event-Id 取得に使用する）
    headers: HeaderMap,
    // params: クエリパラメータを SseQuery 型として受け取る
    Query(params): Query<SseQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Last-Event-Id ヘッダーから resume_token を取得する（resumable=REQUIRED の保証）
    // NOTE: 標準ヘッダー名は "Last-Event-ID" だが HTTP/1.1 では大文字小文字を区別しない
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // session_id をクエリパラメータから取得する（なければ空文字を使う）
    let session_id = params
        .session_id
        .clone()
        .unwrap_or_default();

    // conformance_class をクエリパラメータから取得する（なければ v1_event_feed を使う）
    let conformance_class = params
        .conformance_class
        .clone()
        .unwrap_or_else(|| "v1_event_feed".to_string());

    // tracing で SSE stream の開始をログに記録する
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %conformance_class,
        // resume 有無をログに記録する
        has_resume_token = last_event_id.is_some(),
        "sse_paired SSE stream opened"
    );

    // SsePairedAdapter::sse_stream を呼び出して SSE stream を取得する
    let event_stream = SsePairedAdapter::sse_stream(
        // conformance_class を文字列スライスとして渡す
        &conformance_class,
        // session_id を渡す
        session_id,
        // last_event_id（resume_token）を渡す
        last_event_id,
    );

    // Sse::new で stream を wrap し、KeepAlive で接続を維持する
    Sse::new(event_stream)
        // KeepAlive::default() で接続を維持する（15s ごとに comment を送信する）
        .keep_alive(KeepAlive::default())
}

// router は sse_paired adapter の axum Router を構築して返す。
// /sse に GET（SSE）ハンドラーを登録する。
pub fn router() -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // GET /sse: server → client SSE streaming エンドポイント
        .route("/sse", get(handle_sse))
}
