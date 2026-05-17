// long_poll.rs — spec 01 Bidi §adapter long_poll
// fetch long-poll adapter（v1_event_feed のみサポート）。
// lag≤5000ms target、reconnect 時に resume_token を再発行する。
// 厳格な lag 要件（v1_alert lag≤200ms）や複雑な ordering は担えないため v1_event_feed 専用。

use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
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
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }

    // poll_timeout は long-poll のデフォルトタイムアウトを返す（30s）
    pub fn poll_timeout() -> Duration {
        // spec §constraints の timeout_sec=30 に基づくデフォルト値
        Duration::from_secs(30)
    }
}
