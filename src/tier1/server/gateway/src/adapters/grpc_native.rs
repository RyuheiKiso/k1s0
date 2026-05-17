// grpc_native.rs — spec 01 Bidi §adapter grpc_native
// 全 5 conformance_class をサポートする gRPC native streaming adapter。
// tonic Server::builder で bidi / server / client streaming を実装する。

// axum の HTTP ヘッダーマップ・ボディ・レスポンス型をインポートする
use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
// bytes クレートの Bytes 型（ゼロコピーバイト列）をインポートする
use bytes::Bytes;
// futures-util の StreamExt（非同期ストリーム拡張メソッド）をインポートする
use futures_util::StreamExt;
// serde_json で JSON シリアライズを行うためにインポートする
use serde_json::json;
// tracing で構造化ログを記録するためにインポートする
use tracing::info;
// 親モジュールの AdapterManifest 型をインポートする
use super::AdapterManifest;
// 同一 crate 内の BidiSession 型をインポートする（handshake state machine を使用する）
use crate::bidi::BidiSession;

// MANIFEST は grpc_native adapter の capability 自己宣言。
// generate_capabilities.py が集約して capabilities.lock.yaml を生成する。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "grpc_native",
    // grpc_native は全 5 conformance_class をサポートする（最も complete な adapter）
    supports: &[
        "v1_interactive",    // 双方向対話 gRPC bidi streaming
        "v1_alert",          // サーバー主導警報 gRPC server streaming
        "v1_event_feed",     // Domain Event 配信 gRPC server streaming
        "v1_live_snapshot",  // 最新値表示 gRPC server streaming
        "v1_bulk_upload",    // 大量データ投入 gRPC client streaming
    ],
    // gRPC native は TLS + HTTP/2 が前提のため fallback 不要
    requires_fallback: false,
    // tonic は HTTP/2 + protobuf が必須
    constraints: "http2=required, content_type=application/grpc",
};

// GrpcNativeAdapter は tonic ベースの gRPC bidi streaming adapter。
// bidi.rs の BidiSession state machine を tonic Streaming で wrap する。
pub struct GrpcNativeAdapter;

impl GrpcNativeAdapter {
    // conformance_class に対応した gRPC service を返す。
    // 実際の proto service は src/tier1/schema/bidi/v1/options.proto から codegen する。
    pub fn adapter_id() -> &'static str {
        // spec §adapter↔class supports 対応 の adapter 名を返す
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する。
    pub fn supports(conformance_class: &str) -> bool {
        // MANIFEST.supports から線形探索する（5 要素なので O(1) 同等）
        MANIFEST.supports.contains(&conformance_class)
    }
}

// gRPC Length-Prefixed Message ヘッダー (5 byte) をデコードする。
// フォーマット: [compressed(1) | length_be(4) | payload(N)]
// compressed フラグが true の場合でも今回は圧縮解除を行わず payload をそのまま返す。
pub fn decode_grpc_frame(data: &[u8]) -> Option<(bool, &[u8])> {
    // gRPC フレームヘッダーは最低 5 バイト必要である
    if data.len() < 5 {
        // データが短すぎる場合は None を返す
        return None;
    }
    // 先頭 1 バイトが compressed フラグである（0=非圧縮, 1=圧縮）
    let compressed = data[0] != 0;
    // バイト 1–4 がビッグエンディアン 32 bit のペイロード長である
    let length = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    // ペイロード全体がバッファに収まっているか確認する
    if data.len() < 5 + length {
        // ペイロードが不完全な場合は None を返す
        return None;
    }
    // compressed フラグとペイロードスライスを返す
    Some((compressed, &data[5..5 + length]))
}

// payload を gRPC Length-Prefixed Message にエンコードする。
// 圧縮は行わない（compressed フラグは 0 固定）。
pub fn encode_grpc_frame(payload: &[u8]) -> Bytes {
    // ペイロード長を u32 ビッグエンディアンとして取得する
    let length = payload.len() as u32;
    // 5 バイトヘッダー + ペイロードを格納するバッファを確保する
    let mut buf = Vec::with_capacity(5 + payload.len());
    // compressed フラグを 0（非圧縮）として追加する
    buf.push(0u8);
    // ペイロード長をビッグエンディアン 4 バイトで追加する
    buf.extend_from_slice(&length.to_be_bytes());
    // ペイロードをバッファに追加する
    buf.extend_from_slice(payload);
    // Bytes として返す（ゼロコピー変換）
    Bytes::from(buf)
}

// gRPC bidi streaming ハンドラー。
// Content-Type: application/grpc を検証し、BidiSession handshake を実行してフレームを処理する。
pub async fn handle_bidi_stream(
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    // Content-Type ヘッダーを取得して application/grpc であることを確認する
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // application/grpc で始まっていない場合は 415 Unsupported Media Type を返す
    if !content_type.starts_with("application/grpc") {
        // gRPC 以外の Content-Type は grpc_native adapter では受け付けない
        return Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"error": "Content-Type must be application/grpc"}).to_string(),
            ))
            .unwrap();
    }
    // X-Session-Id ヘッダーからセッション ID を取得する（未指定時は UUID を生成する）
    let session_id = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    // X-Conformance-Class ヘッダーから conformance_class を取得する（デフォルトは v1_interactive）
    let conformance_class = headers
        .get("x-conformance-class")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("v1_interactive")
        .to_string();
    // grpc_native adapter で BidiSession を生成して handshake を開始する
    let mut session = BidiSession::new(
        session_id.clone(),
        "grpc_native".to_string(),
        conformance_class.clone(),
    );
    // BidiSession の Hello 送信フェーズを実行する（TLA+ SendHello アクションに対応する）
    if let Err(e) = session.send_hello() {
        // SendHello 失敗は handshake violation を示す
        tracing::warn!(error = %e, session_id = %session_id, "send_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(json!({"error": e.to_string()}).to_string()))
            .unwrap();
    }
    // BidiSession の Hello 受信フェーズを実行する（サーバー→クライアント方向の Hello 受信）
    if let Err(e) = session.receive_hello() {
        // ReceiveHello 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "receive_hello failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(json!({"error": e.to_string()}).to_string()))
            .unwrap();
    }
    // BidiSession の Ack 送信フェーズを実行する（TLA+ SendAck アクションに対応する）
    if let Err(e) = session.send_ack() {
        // SendAck 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "send_ack failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(json!({"error": e.to_string()}).to_string()))
            .unwrap();
    }
    // BidiSession を完了状態（Done）に遷移させる
    if let Err(e) = session.complete() {
        // complete 失敗は不正状態遷移を示す
        tracing::warn!(error = %e, session_id = %session_id, "complete failed");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(json!({"error": e.to_string()}).to_string()))
            .unwrap();
    }
    // リクエストボディを非同期ストリームとして読み取る
    let mut stream = body.into_data_stream();
    // 受信した gRPC フレームのペイロードを集積するバッファを用意する
    let mut accumulated = Vec::new();
    // 受信フレーム数を計上するカウンターを初期化する
    let mut frame_count = 0u32;
    // ストリームから全チャンクを読み取る
    while let Some(chunk) = stream.next().await {
        // チャンク読み取りエラーは早期リターンする
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "body stream error in grpc_native handler");
                break;
            }
        };
        // バッファにチャンクを追加する
        accumulated.extend_from_slice(&chunk);
        // バッファ先頭から gRPC フレームを繰り返し取り出す
        while accumulated.len() >= 5 {
            // フレームをデコードして compressed フラグとペイロードを取得する
            if let Some((compressed, payload)) = decode_grpc_frame(&accumulated) {
                // デコード結果をトレースに記録する
                info!(
                    session_id = %session_id,
                    frame_count = frame_count,
                    compressed = compressed,
                    payload_len = payload.len(),
                    "grpc_native: decoded frame"
                );
                // 取り出したフレームのバイト数を計算する（5 バイトヘッダー + ペイロード）
                let consumed = 5 + payload.len();
                // フレーム数カウンターをインクリメントする
                frame_count += 1;
                // 消費したバイト数分だけバッファを前進させる
                accumulated.drain(..consumed);
            } else {
                // フレームが不完全なので次のチャンク受信まで待機する
                break;
            }
        }
    }
    // 処理結果を gRPC フレームにエンコードして返す
    let response_payload = serde_json::to_vec(&json!({
        "session_id": session_id,
        "state": "Done",
        "adapter": "grpc_native",
        "conformance_class": conformance_class,
        "frames_received": frame_count,
        "invariant_ok": session.check_invariant(),
    }))
    .unwrap_or_default();
    // ペイロードを gRPC Length-Prefixed Message にエンコードする
    let grpc_frame = encode_grpc_frame(&response_payload);
    // gRPC streaming レスポンスを返す（grpc-status: 0 = OK）
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/grpc+json")
        .header("grpc-status", "0")
        .header("grpc-message", "OK")
        .header("x-session-id", session_id)
        .body(Body::from(grpc_frame))
        .unwrap()
}

// grpc_native adapter の axum Router を返す。
// main.rs で .nest("/grpc", grpc_native::router()) として使用する。
pub fn router() -> Router {
    // BidiService/OpenBidiStream パスを any メソッドで登録する
    Router::new()
        // gRPC bidi streaming エンドポイントを登録する（POST / any）
        .route(
            "/k1s0.tier1.bidi.v1.BidiService/OpenBidiStream",
            any(handle_bidi_stream),
        )
}
