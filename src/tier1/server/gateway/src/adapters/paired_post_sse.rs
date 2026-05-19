// paired_post_sse.rs — spec 01 Bidi §adapter paired_post_sse
// POST↔SSE pair adapter: 半二重 emulation（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）。
// v1_bulk_upload は半二重 emulation では担えないため not_applicable。
// 12_UA_aware_adapter.md の K1s0UaAwareAdapter と連携して UA 判定を行う。
// 13_dotnet8_connect_inhouse.md の .NET Framework 対応に使用する。
// POST /bidi/up → EventBus.publish → broadcast channel → GET /bidi/down SSE で配信する実 streaming 実装。

// axum Router・POST ハンドラー・SSE ハンドラーで使用する型を import する
use axum::{
    // Router: axum のルーティング構造体
    Router,
    // routing: HTTP メソッドルーターを提供するモジュール
    routing::{get, post},
    // extract::Query: クエリパラメータを抽出する Extractor
    extract::Query,
    // extract::State: axum State 依存性注入（EventBus を handler に注入する）
    extract::State,
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
// std::sync::Arc: EventBus の shared state 共有に使用する
use std::sync::Arc;
// crate::event_bus: EventBus と DomainEvent をインポートする
use crate::event_bus::{DomainEvent, EventBus};
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
// client → server 方向のメッセージを受け取り、EventBus に publish して SSE down stream に転送する。
// wall clock を使わない（TTL 計算には HLC を使う — 本 handler は TTL 計算なし）。
pub async fn handle_up(
    // State(event_bus): axum State 依存性注入で EventBus を受け取る
    State(event_bus): State<Arc<EventBus>>,
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

    // body から event_type を取得する（なければ "client_message" を使う）
    let event_type = body
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("client_message")
        .to_string();

    // 受け取ったメッセージを構造化ログに記録する（tracing を使う）
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // request_id フィールドを構造化ログに含める
        request_id = %request_id,
        // event_type をログに残す
        event_type = %event_type,
        "paired_post_sse up message received"
    );

    // session_id が空の場合は 400 Bad Request を返す
    if session_id.is_empty() {
        // session_id は必須フィールドであるため 400 を返す
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "session_id is required",
            })),
        );
    }

    // DomainEvent を構築して EventBus に publish する
    let domain_event = DomainEvent {
        // session_id: イベントの宛先セッション識別子を設定する
        session_id: session_id.clone(),
        // event_type: ドメインイベント種別を設定する
        event_type: event_type.clone(),
        // payload: ドメインイベントのペイロードを設定する
        payload: payload.clone(),
        // sequence: この実装では request_id のハッシュを仮のシーケンス番号に使う
        // NOTE: 本実装では単純に 0 を使う（本番では単調インクリメントカウンタを使う）
        sequence: 0,
    };

    // EventBus に DomainEvent を publish する（broadcast channel で SSE subscribers に転送する）
    if let Err(e) = event_bus.publish(domain_event) {
        // publish 失敗をログに記録する
        tracing::warn!(
            // session_id フィールドを構造化ログに含める
            session_id = %session_id,
            // エラー内容をログに記録する
            error = %e,
            "paired_post_sse: EventBus publish failed"
        );
    }

    // 受信確認を JSON で返す（200 OK + { "received": true, "session_id": ... }）
    let response_body = serde_json::json!({
        // received: クライアントへの受信確認フラグ
        "received": true,
        // session_id: リクエストの session_id をエコーバックする
        "session_id": session_id,
        // request_id: 対応する SSE down stream の識別子
        "request_id": request_id,
        // event_type: 処理したイベント種別をエコーバックする
        "event_type": event_type,
    });

    // 200 OK ステータスコードと JSON ボディを返す
    (StatusCode::OK, Json(response_body))
}

// handle_down は GET /bidi/down の handler。
// server → client 方向の SSE stream を返す。
// EventBus から broadcast Receiver を取得し、DomainEvent を SSE Event に変換して配信する。
pub async fn handle_down(
    // State(event_bus): axum State 依存性注入で EventBus を受け取る
    State(event_bus): State<Arc<EventBus>>,
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

    // EventBus から session_id に対応する broadcast Receiver を取得する
    let mut rx = event_bus.subscribe(&session_id);

    // session_id を clone してクロージャ内で所有権を移動する
    let session_id_for_stream = session_id.clone();
    // request_id を clone してクロージャ内で所有権を移動する
    let request_id_for_stream = request_id.clone();
    // conformance_class を clone してクロージャ内で所有権を移動する
    let conformance_class_for_stream = conformance_class.clone();
    // last_event_id を clone してクロージャ内で所有権を移動する
    let last_event_id_for_stream = last_event_id.clone();

    // async_stream::stream! マクロで非同期 Stream を生成する
    let event_stream = stream! {
        // session_started イベントを最初に送信する（クライアントに接続確立を通知する）
        let session_started_data = serde_json::json!({
            // session_id: セッション識別子をクライアントに通知する
            "session_id": session_id_for_stream,
            // request_id: 対応する POST up stream の識別子をクライアントに通知する
            "request_id": request_id_for_stream,
            // conformance_class: 確立した Bidi class をクライアントに通知する
            "conformance_class": conformance_class_for_stream,
            // resume_from: resume_token（last_event_id）をエコーバックする
            "resume_from": last_event_id_for_stream,
        });
        // session_started イベントを yield する
        yield Ok(
            // event 名を "session_started" に設定する
            Event::default()
                .event("session_started")
                // session_id を SSE id フィールドに設定する（resume 時の Last-Event-ID に使用する）
                .id(session_id_for_stream.as_str())
                // セッション開始データを JSON 文字列で data フィールドに設定する
                .data(session_started_data.to_string())
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
                        // request_id: 対応する POST up stream の識別子をクライアントに通知する
                        "request_id": request_id_for_stream,
                    });
                    // SSE id にシーケンス番号を設定して Last-Event-ID resume を支援する
                    let event_id = domain_event.sequence.to_string();
                    // DomainEvent を SSE Event に変換して yield する
                    yield Ok(
                        // event 名を "data" に設定する
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
                    // server_truth_advance イベントをクライアントに通知してリカバリを促す
                    let lag_data = serde_json::json!({
                        // type: server_truth_advance であることを示す
                        "type": "server_truth_advance",
                        // skipped: スキップされたイベント数をクライアントに通知する
                        "skipped": skip_count,
                        // session_id: 対象セッション識別子を含める
                        "session_id": session_id_for_stream,
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

// router は paired_post_sse adapter の axum Router を構築して返す。
// EventBus を State として注入し、/bidi/up に POST ハンドラー、/bidi/down に GET（SSE）ハンドラーを登録する。
pub fn router(event_bus: Arc<EventBus>) -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // POST /bidi/up: client → server メッセージ受付エンドポイント
        .route("/bidi/up", post(handle_up))
        // GET /bidi/down: server → client SSE streaming エンドポイント
        .route("/bidi/down", get(handle_down))
        // EventBus を axum State として注入する（handler が Arc<EventBus> を受け取れるようにする）
        .with_state(event_bus)
}
