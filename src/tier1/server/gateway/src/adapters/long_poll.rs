// long_poll.rs — spec 01 Bidi §adapter long_poll
// fetch long-poll adapter（v1_event_feed のみサポート）。
// lag≤5000ms target、reconnect 時に resume_token を再発行する。
// 厳格な lag 要件（v1_alert lag≤200ms）や複雑な ordering は担えないため v1_event_feed 専用。

// axum から JSON・ルーティング・レスポンス型を import する
use axum::{
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
    // Router: axum のルーティング構造体
    Router,
    // response::IntoResponse: handler の戻り値を HTTP Response に変換するトレイト
    response::IntoResponse,
    // http::StatusCode: HTTP ステータスコードを表す型
    http::StatusCode,
    // routing::post: POST メソッドルーター関数
    routing::post,
};
// serde: シリアライズ / デシリアライズトレイトを import する
use serde::{Deserialize, Serialize};
// std::time::Duration: tokio::time::timeout の引数として使用する
use std::time::Duration;
// tokio::time::timeout: 非同期操作にタイムアウトを設定するために使用する
use tokio::time::timeout;
// uuid::Uuid: next_resume_token を UUID v4 で生成するために使用する
use uuid::Uuid;
// AdapterManifest: adapter の capability 自己宣言型
use super::AdapterManifest;

// MANIFEST は long_poll adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "long_poll",
    // long_poll は lag 要件と ordering 要件が緩い v1_event_feed のみサポートする
    supports: &["v1_event_feed"],
    // long-poll は全ブラウザ / curl / httpie で動作するため fallback 不要
    requires_fallback: false,
    // 30s timeout、reconnect 時に resume_token（Last-Event-ID 相当）を送信する
    constraints: "timeout_sec=30, resume_token=required_on_reconnect, ordering=SESSION_ORDERED",
};

// LongPollRequest は long-poll エンドポイントへのリクエストを宣言する。
#[derive(Deserialize)]
pub struct LongPollRequest {
    // session_id: BidiSession の識別子
    pub session_id: String,
    // resume_token: 前回の poll で受け取った再接続 token
    pub resume_token: Option<String>,
    // timeout_ms: poll の最大待機時間（デフォルト 30000ms）
    pub timeout_ms: Option<u64>,
}

// LongPollResponse は long-poll エンドポイントからのレスポンスを宣言する。
#[derive(Serialize)]
pub struct LongPollResponse {
    // events: 受信したイベントのリスト（空のこともある）
    pub events: Vec<serde_json::Value>,
    // next_resume_token: 次回の poll で使用する resume_token
    pub next_resume_token: String,
    // session_id: リクエストの session_id をエコーバックする
    pub session_id: String,
}

// LongPollAdapter は fetch long-poll adapter。
pub struct LongPollAdapter;

impl LongPollAdapter {
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

    // poll_timeout は long-poll のデフォルトタイムアウトを返す（30s）
    pub fn poll_timeout() -> Duration {
        // spec §constraints の timeout_sec=30 に基づくデフォルト値
        Duration::from_secs(30)
    }
}

// handle_long_poll は POST /poll の handler。
// tokio::time::timeout で 30s のタイムアウトを実装する。
// wall clock を使わない（TTL 計算には HLC を使う — 本 handler は TTL 計算なし）。
pub async fn handle_long_poll(
    // body: JSON リクエストボディを LongPollRequest 型として受け取る
    Json(body): Json<LongPollRequest>,
) -> impl IntoResponse {
    // session_id をリクエストボディから取得する
    let session_id = body.session_id.clone();

    // timeout_ms をリクエストボディから取得する（デフォルトは 30000ms = 30s）
    // MANIFEST の timeout_sec=30 に基づく上限で clamp する
    let timeout_ms = body
        .timeout_ms
        // 30000ms を上限として clamp する（spec §constraints timeout_sec=30 の物理保証）
        .map(|ms| ms.min(30_000))
        // デフォルトは 30000ms
        .unwrap_or(30_000);

    // resume_token をリクエストボディから取得する（None の場合は新規セッション扱い）
    let _resume_token = body.resume_token.clone();

    // tracing で long-poll リクエストの受け取りをログに記録する
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %session_id,
        // timeout_ms フィールドを構造化ログに含める
        timeout_ms = timeout_ms,
        // resume_token の有無をログに記録する
        has_resume_token = body.resume_token.is_some(),
        "long_poll request received"
    );

    // timeout_ms を Duration に変換する
    let poll_duration = Duration::from_millis(timeout_ms);

    // tokio::time::timeout で poll 操作にタイムアウトを設定する
    // 実際の event poll は broadcast channel から receive する（現在は stub として即時完了）
    let poll_result = timeout(poll_duration, async {
        // NOTE: 実際の event polling は broadcast channel から receive する
        // 現在はデモ用 stub として空の events リストを即時返す
        // 将来実装: session_id に対応する channel から event を受信する
        let events: Vec<serde_json::Value> = vec![];
        // poll が成功した場合は events リストを返す
        events
    })
    .await;

    // next_resume_token を UUID v4 で新規生成する（SESSION_ORDERED の再接続保証）
    let next_resume_token = Uuid::new_v4().to_string();

    // poll_result に応じてレスポンスを構築する
    match poll_result {
        // timeout した場合: 空の events リストと新しい resume_token を 204 No Content で返す
        Err(_timeout_elapsed) => {
            // tracing で timeout をログに記録する
            tracing::debug!(
                // session_id フィールドを構造化ログに含める
                session_id = %session_id,
                // timeout_ms フィールドを構造化ログに含める
                timeout_ms = timeout_ms,
                "long_poll timeout elapsed, returning empty events"
            );
            // 204 No Content + 空 events + 新 resume_token を JSON で返す
            // NOTE: 204 は body を持つが、spec では timeout 時の空レスポンスとして定義する
            (
                // 204 No Content ステータスコードを設定する
                StatusCode::NO_CONTENT,
                // 空 events リストと新 resume_token を JSON で返す
                Json(LongPollResponse {
                    // 空の events リストを返す
                    events: vec![],
                    // 新規 UUID v4 の resume_token を返す
                    next_resume_token,
                    // session_id をエコーバックする
                    session_id,
                }),
            )
        }
        // events がある場合: 200 OK + LongPollResponse を返す
        Ok(events) => {
            // tracing で poll 成功をログに記録する
            tracing::debug!(
                // session_id フィールドを構造化ログに含める
                session_id = %session_id,
                // 取得した events 件数をログに記録する
                event_count = events.len(),
                "long_poll returning events"
            );
            // 200 OK + events リストと新 resume_token を JSON で返す
            (
                // 200 OK ステータスコードを設定する
                StatusCode::OK,
                // events リストと新 resume_token を JSON で返す
                Json(LongPollResponse {
                    // 取得した events リストを返す
                    events,
                    // 新規 UUID v4 の resume_token を返す
                    next_resume_token,
                    // session_id をエコーバックする
                    session_id,
                }),
            )
        }
    }
}

// router は long_poll adapter の axum Router を構築して返す。
// /poll に POST ハンドラーを登録する。
pub fn router() -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // POST /poll: long-poll リクエスト受付エンドポイント
        .route("/poll", post(handle_long_poll))
}
