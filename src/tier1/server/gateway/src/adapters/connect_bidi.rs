// connect_bidi.rs — spec 01 Bidi §adapter connect_bidi
// 全 5 conformance_class をサポートする Connect-RPC bidi adapter。
// fetch full-duplex streams を使用し、UA subclass 別に adapter を選択する。
// 13_dotnet8_connect_inhouse.md の Connect-RPC inhouse 実装と連携する。
// Connect-RPC length-prefixed frame decode/encode + EventBus broadcast + BidiSession 状態遷移を実装する。

// axum の JSON extractor・レスポンス型・ルーター・ルーティング関数をインポートする
use axum::{
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
    // Router: axum のルーティング構造体
    Router,
    // http::HeaderMap: HTTP リクエストヘッダーを受け取る型
    http::{HeaderMap, StatusCode},
    // response::IntoResponse: 各 handler の戻り値を HTTP Response に変換するトレイト
    response::{IntoResponse, Response},
    // routing::post: POST メソッドルーター関数
    routing::post,
    // body::Body: HTTP レスポンスボディ型
    body::Body,
    // extract::State: axum State 依存性注入（EventBus を handler に注入する）
    extract::State,
};
// bytes::Bytes: ゼロコピーバイト操作（Connect-RPC frame decode に使用する）
use bytes::Bytes;
// serde で JSON デシリアライズを行うためにインポートする
use serde::Deserialize;
// serde_json で JSON オブジェクトを構築するためにインポートする
use serde_json::json;
// tracing で構造化ログを記録するためにインポートする
use tracing::info;
// std::sync::Arc: EventBus の shared state 共有に使用する
use std::sync::Arc;
// crate::event_bus: EventBus と DomainEvent をインポートする
use crate::event_bus::{DomainEvent, EventBus};
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

// ConnectFrame は Connect-RPC の length-prefixed frame を宣言する。
// フォーマット: [flags: 1 byte][length: 4 bytes big-endian][payload: length bytes]
// flags: 0x00 = data frame, 0x01 = end-of-stream frame, 0x02 = trailers frame
pub struct ConnectFrame {
    // flags: frame の種別フラグ（0x00=data, 0x01=eos, 0x02=trailers）
    pub flags: u8,
    // payload: frame のペイロードバイト列
    pub payload: Bytes,
}

impl ConnectFrame {
    // END_OF_STREAM_FLAG: ストリーム終端を示す flags 値
    pub const END_OF_STREAM_FLAG: u8 = 0x01;
    // TRAILERS_FLAG: トレーラーフレームを示す flags 値
    pub const TRAILERS_FLAG: u8 = 0x02;

    // decode は Connect-RPC length-prefixed frame をバイト列からデコードする。
    // フォーマット: [flags: 1 byte][length: 4 bytes big-endian][payload: length bytes]
    // 不正なフォーマットの場合は Err を返す（呼び出し元が 400 を返す）。
    pub fn decode(buf: &[u8]) -> Result<Self, &'static str> {
        // フレームの最小サイズ（flags 1 byte + length 4 bytes = 5 bytes）を確認する
        if buf.len() < 5 {
            // バッファが小さすぎる場合はエラーを返す
            return Err("buffer too small: minimum 5 bytes required");
        }
        // flags バイトを読み取る（先頭 1 byte）
        let flags = buf[0];
        // length フィールドを big-endian でデコードする（次の 4 bytes）
        let length = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        // ペイロードのバイト数が実際のバッファサイズと一致するか確認する
        if buf.len() < 5 + length {
            // ペイロードが不足している場合はエラーを返す
            return Err("buffer too small: payload length exceeds buffer");
        }
        // ペイロードを Bytes としてコピーする（5 byte ヘッダーをスキップする）
        let payload = Bytes::copy_from_slice(&buf[5..5 + length]);
        // ConnectFrame を返す
        Ok(Self { flags, payload })
    }

    // encode は ConnectFrame を length-prefixed バイト列にエンコードする。
    // フォーマット: [flags: 1 byte][length: 4 bytes big-endian][payload]
    pub fn encode(&self) -> Vec<u8> {
        // 出力バッファを確保する（5 byte ヘッダー + ペイロード長）
        let mut buf = Vec::with_capacity(5 + self.payload.len());
        // flags バイトを書き込む
        buf.push(self.flags);
        // ペイロード長を big-endian で書き込む（4 bytes）
        let length = (self.payload.len() as u32).to_be_bytes();
        // length バイト列をバッファに追加する
        buf.extend_from_slice(&length);
        // ペイロードをバッファに追加する
        buf.extend_from_slice(&self.payload);
        // エンコード済みバッファを返す
        buf
    }

    // is_end_of_stream は END_OF_STREAM_FLAG が設定されているかを確認する
    pub fn is_end_of_stream(&self) -> bool {
        // flags に END_OF_STREAM_FLAG が含まれているか確認する
        self.flags & Self::END_OF_STREAM_FLAG != 0
    }
}

// Connect-RPC streaming リクエストのボディ構造体。
// Connect プロトコルでは JSON または proto3 のいずれかで送信される。
#[derive(Debug, Deserialize)]
pub struct ConnectStreamRequest {
    // session_id: クライアントが生成した UUID v4
    pub session_id: Option<String>,
    // 対象の conformance_class（未指定時は v1_interactive をデフォルトとする）
    pub conformance_class: Option<String>,
    // event_type: publish するイベント種別（省略時は "connect_stream" を使う）
    pub event_type: Option<String>,
    // payload: EventBus に publish するペイロード（省略時は null）
    pub payload: Option<serde_json::Value>,
    // raw_frame_hex: length-prefixed frame の raw hex データ（直接 frame decode に使用する）
    pub raw_frame_hex: Option<String>,
}

// ConnectBidiState は connect_bidi handler に注入する共有状態を宣言する。
pub struct ConnectBidiState {
    // event_bus: DomainEvent を broadcast channel で配信する
    pub event_bus: Arc<EventBus>,
}

// Connect-RPC streaming ハンドラー。
// POST /connect/bidi/stream を受け付け、BidiSession handshake を実行して結果を返す。
// Connect-RPC length-prefixed frame を decode して EventBus に publish する。
pub async fn handle_connect_stream(
    // State(state): axum State 依存性注入で ConnectBidiState を受け取る
    State(state): State<Arc<ConnectBidiState>>,
    // headers: HTTP リクエストヘッダーを受け取る（Content-Type 検証に使用する）
    headers: HeaderMap,
    // body: JSON リクエストボディを受け取る（session_id / conformance_class を取得する）
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
    // event_type を取得する（未指定時は "connect_stream" を使う）
    let event_type = body.event_type.unwrap_or_else(|| "connect_stream".to_string());
    // payload を取得する（未指定時は null を使う）
    let payload = body.payload.unwrap_or(serde_json::Value::Null);

    // raw_frame_hex が指定されている場合は Connect-RPC frame を decode する
    if let Some(hex_data) = body.raw_frame_hex {
        // hex データをバイト列にデコードする
        match hex::decode(&hex_data) {
            Ok(raw_bytes) => {
                // ConnectFrame::decode で length-prefixed frame をデコードする
                match ConnectFrame::decode(&raw_bytes) {
                    Ok(frame) => {
                        // frame decode 成功をログに記録する
                        tracing::debug!(
                            // session_id フィールドを構造化ログに含める
                            session_id = %session_id,
                            // flags フィールドを構造化ログに含める
                            flags = frame.flags,
                            // payload サイズをログに記録する
                            payload_len = frame.payload.len(),
                            "connect_bidi: frame decoded"
                        );
                        // END_OF_STREAM フレームの場合はストリームを終了する
                        if frame.is_end_of_stream() {
                            // EOS フレームをログに記録する
                            tracing::info!(
                                session_id = %session_id,
                                "connect_bidi: END_OF_STREAM frame received"
                            );
                        }
                    }
                    // frame decode 失敗は 400 Bad Request を返す
                    Err(e) => {
                        // frame decode エラーをログに記録する
                        tracing::warn!(error = %e, "connect_bidi: frame decode failed");
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header("content-type", "application/connect+json")
                            .body(Body::from(
                                json!({
                                    "code": "invalid_argument",
                                    "message": format!("Connect frame decode failed: {e}")
                                })
                                .to_string(),
                            ))
                            .unwrap();
                    }
                }
            }
            // hex デコード失敗は 400 Bad Request を返す
            Err(e) => {
                // hex デコードエラーをログに記録する
                tracing::warn!(error = %e, "connect_bidi: hex decode of raw_frame_hex failed");
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/connect+json")
                    .body(Body::from(
                        json!({
                            "code": "invalid_argument",
                            "message": format!("hex decode failed: {e}")
                        })
                        .to_string(),
                    ))
                    .unwrap();
            }
        }
    }

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

    // EventBus に DomainEvent を publish する（broadcast channel で subscribers に転送する）
    let domain_event = DomainEvent {
        // session_id: イベントの宛先セッション識別子を設定する
        session_id: session_id.clone(),
        // event_type: ドメインイベント種別を設定する
        event_type: event_type.clone(),
        // payload: ドメインイベントのペイロードを設定する
        payload: payload.clone(),
        // sequence: recv_count をシーケンス番号として使用する
        sequence: session.recv_count as u64,
    };
    // EventBus に DomainEvent を publish する
    if let Err(e) = state.event_bus.publish(domain_event) {
        // publish 失敗をログに記録する（非致命的エラー）
        tracing::warn!(
            // session_id フィールドを構造化ログに含める
            session_id = %session_id,
            // エラー内容をログに記録する
            error = %e,
            "connect_bidi: EventBus publish failed (non-fatal)"
        );
    }

    // セッション完了をトレースに記録する
    info!(
        session_id = %session_id,
        conformance_class = %conformance_class,
        state = "Done",
        invariant_ok = session.check_invariant(),
        recv_count = session.recv_count,
        send_count = session.send_count,
        "connect_bidi: handshake completed"
    );

    // recv_count をレスポンスに含める（Connect-RPC spec の streaming state 要件）
    let recv_count = session.recv_count;
    // send_count をレスポンスに含める
    let send_count = session.send_count;

    // BidiSession の状態を Connect-RPC 形式の JSON レスポンスとして返す
    let response_body = json!({
        "session_id": session_id,
        "state": "Done",
        "adapter": "connect_bidi",
        "conformance_class": conformance_class,
        "send_count": send_count,
        "recv_count": recv_count,
        "invariant_ok": session.check_invariant(),
        // event_published: EventBus に publish したことを示す
        "event_published": true,
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
// EventBus を State として注入し、/connect/bidi/stream に Connect-RPC streaming ハンドラーを登録する。
pub fn router(event_bus: Arc<EventBus>) -> Router {
    // ConnectBidiState を生成して axum State に注入する
    let state = Arc::new(ConnectBidiState { event_bus });
    // POST /connect/bidi/stream に Connect-RPC streaming ハンドラーを登録する
    Router::new()
        // Connect-RPC streaming エンドポイントを POST として登録する
        .route("/bidi/stream", post(handle_connect_stream))
        // ConnectBidiState を axum State として注入する
        .with_state(state)
}
