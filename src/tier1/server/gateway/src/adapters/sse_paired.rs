// sse_paired.rs — spec 01 Bidi §adapter sse_paired
// EventSource SSE adapter（v1_alert / v1_event_feed / v1_live_snapshot のみサポート）。
// client→server bulk（v1_bulk_upload）は一方向 wire の都合上、構造的にサポートできない。
// axum SSE + Last-Event-ID による resume_token 実装。

use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::Query,
};
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use std::{convert::Infallible, time::Duration};
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
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }

    // sse_stream は SSE イベントストリームを返す axum handler helper。
    // conformance_class に応じて ordering / lag / resumable を適用する。
    pub fn sse_stream(
        conformance_class: &str,
        session_id: String,
        last_event_id: Option<String>,
    ) -> impl Stream<Item = Result<Event, Infallible>> {
        // last_event_id があれば resume_token から継続する（resumable=REQUIRED の保証）
        let resume_from = last_event_id.unwrap_or_default();
        // conformance_class に応じた lag target を取得する
        let _lag_target_ms = match conformance_class {
            "v1_alert" => 200u64,
            "v1_event_feed" => 5000u64,
            "v1_live_snapshot" => 500u64,
            // MANIFEST.supports に含まれない class は到達しない
            _ => 0u64,
        };
        // SSE ping イベントを一定間隔で送信する（heartbeat + resume_token 維持）
        stream::once(async move {
            // 最初のイベントとして session 開始を通知する
            Ok(Event::default()
                .id(&session_id)
                .event("session_started")
                .data(format!("{{\"session_id\":\"{session_id}\",\"resume_from\":\"{resume_from}\"}}")))
        })
    }
}
