// sse_paired.rs — spec 01 Bidi §adapter sse_paired
// EventSource SSE adapter（v1_alert / v1_event_feed / v1_live_snapshot のみサポート）。
// client→server bulk（v1_bulk_upload）は一方向 wire の都合上、構造的にサポートできない。
// axum SSE + Last-Event-ID による resume_token 実装。
// EventBus（broadcast channel）から受信したイベントを SSE で配信する実 streaming 実装。

// axum から SSE・ルーティング・抽出・レスポンス型・State を import する
use axum::{
    // Router: axum のルーティング構造体
    Router,
    // routing::get: GET メソッドルーター関数
    routing::get,
    // extract::Query: クエリパラメータを抽出する Extractor
    extract::Query,
    // extract::State: axum State 依存性注入（EventBus を handler に注入する）
    extract::State,
    // response::sse: SSE 型（Event / KeepAlive / Sse）を提供するモジュール
    response::sse::{Event, KeepAlive, Sse},
    // http::HeaderMap: HTTP リクエストヘッダーを受け取る型
    http::HeaderMap,
};
// async_stream::stream! マクロで非同期 Stream を生成するために import する
use async_stream::stream;
// serde::Deserialize: クエリパラメータ型を QueryString からデシリアライズするために使用する
use serde::Deserialize;
// std::convert::Infallible: SSE Stream の Item エラー型に使用する（stream が失敗しないことを示す）
use std::convert::Infallible;
// std::sync::Arc: EventBus の shared state 共有に使用する
use std::sync::Arc;
// crate::event_bus::EventBus: セッションごとの broadcast channel 管理
use crate::event_bus::EventBus;
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
}

// handle_sse は GET /sse の handler。
// server → client 一方向 SSE streaming エンドポイント。
// Last-Event-Id ヘッダーで resume_token を受け取り、resumable=REQUIRED を保証する。
// EventBus から broadcast Receiver を取得し、DomainEvent を SSE Event に変換して配信する。
pub async fn handle_sse(
    // State(event_bus): axum State 依存性注入で EventBus を受け取る
    State(event_bus): State<Arc<EventBus>>,
    // headers: HTTP リクエストヘッダーを受け取る（Last-Event-Id 取得に使用する）
    headers: HeaderMap,
    // params: クエリパラメータを SseQuery 型として受け取る
    Query(params): Query<SseQuery>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
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

    // resume_from: last_event_id がある場合はその値、なければ空文字を使う
    let resume_from = last_event_id.clone().unwrap_or_default();

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

    // EventBus から session_id に対応する broadcast Receiver を取得する
    let mut rx = event_bus.subscribe(&session_id);

    // session_id を clone してクロージャ内で所有権を移動する
    let session_id_for_stream = session_id.clone();
    // conformance_class を clone してクロージャ内で所有権を移動する
    let conformance_class_for_stream = conformance_class.clone();

    // async_stream::stream! マクロで非同期 Stream を生成する
    let event_stream = stream! {
        // session_started イベントを最初に送信してクライアントに接続確立を通知する
        let started_data = serde_json::json!({
            // session_id: セッション識別子をクライアントに通知する
            "session_id": session_id_for_stream,
            // conformance_class: 確立した Bidi class をクライアントに通知する
            "conformance_class": conformance_class_for_stream,
            // resume_from: resume_token（last_event_id）をエコーバックする
            "resume_from": resume_from,
        });
        // session_started イベントを yield する（接続確立を通知する）
        yield Ok(
            // event 名を "session_started" に設定する
            Event::default()
                .event("session_started")
                // session_id を SSE id フィールドに設定する（Last-Event-ID の基底値として使用する）
                .id(session_id_for_stream.as_str())
                // セッション開始データを JSON 文字列で data フィールドに設定する
                .data(started_data.to_string())
        );

        // broadcast Receiver からイベントを受信し続ける無限ループ
        loop {
            // recv(): broadcast channel からイベントを非同期受信する
            match rx.recv().await {
                // 正常受信: DomainEvent を SSE Event に変換して yield する
                Ok(domain_event) => {
                    // DomainEvent を JSON 文字列にシリアライズする
                    let data = serde_json::json!({
                        // event_type: ドメインイベント種別をクライアントに通知する
                        "event_type": domain_event.event_type,
                        // payload: ドメインイベントのペイロードをクライアントに配信する
                        "payload": domain_event.payload,
                        // sequence: SESSION_ORDERED の担保のためにシーケンス番号を送信する
                        "sequence": domain_event.sequence,
                        // session_id: イベントの宛先セッション識別子をクライアントに通知する
                        "session_id": domain_event.session_id,
                    });
                    // SSE id にシーケンス番号を設定して Last-Event-ID resume を支援する
                    let event_id = domain_event.sequence.to_string();
                    // DomainEvent を SSE Event に変換して yield する
                    yield Ok(
                        // event 名を event_type に設定する
                        Event::default()
                            .event("data")
                            // SSE id フィールドにシーケンス番号を設定する
                            .id(event_id.as_str())
                            // ドメインイベントデータを JSON 文字列で data フィールドに設定する
                            .data(data.to_string())
                    );
                }
                // Lagged: バッファ溢れで一部イベントがスキップされた（broadcast channel が遅延した）
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skip_count)) => {
                    // lagged イベントをクライアントに通知してクライアント側でリカバリを促す
                    let lag_data = serde_json::json!({
                        // type: lagged であることを示す
                        "type": "server_truth_advance",
                        // skipped: スキップされたイベント数をクライアントに通知する
                        "skipped": skip_count,
                    });
                    // server_truth_advance イベントを yield する（クライアントにラグを通知する）
                    yield Ok(
                        // event 名を "server_truth_advance" に設定する
                        Event::default()
                            .event("server_truth_advance")
                            // ラグ通知データを JSON 文字列で data フィールドに設定する
                            .data(lag_data.to_string())
                    );
                    // Lagged 後は受信を継続する（チャンネルは有効なため）
                }
                // Closed: broadcast channel が閉じられた（送信側がドロップした）
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    // complete イベントをクライアントに送信してストリームの終端を通知する
                    yield Ok(
                        // event 名を "complete" に設定する
                        Event::default()
                            .event("complete")
                            // ストリーム終了データを JSON 文字列で data フィールドに設定する
                            .data("{\"reason\":\"channel_closed\"}")
                    );
                    // ループを終了してストリームを閉じる
                    break;
                }
            }
        }
    };

    // Sse::new で stream を wrap し、KeepAlive で接続を維持する
    Sse::new(event_stream)
        // KeepAlive::default() で接続を維持する（15s ごとに comment を送信する）
        .keep_alive(KeepAlive::default())
}

// router は sse_paired adapter の axum Router を構築して返す。
// EventBus を State として注入し、/sse に GET（SSE）ハンドラーを登録する。
pub fn router(event_bus: Arc<EventBus>) -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // GET /sse: server → client SSE streaming エンドポイント
        .route("/sse", get(handle_sse))
        // EventBus を axum State として注入する（handler が Arc<EventBus> を受け取れるようにする）
        .with_state(event_bus)
}
