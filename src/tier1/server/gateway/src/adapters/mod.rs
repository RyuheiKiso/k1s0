// adapters/mod.rs — 8 transport adapter モジュール宣言
// 01_Bidi適合仕様.md §adapter↔class supports 対応 に基づく 8 adapter を宣言する。
// 各 adapter は capability_manifest を宣言し、生成器が capabilities.lock.yaml を更新する。

// grpc_native: tonic bidi server（全 5 class サポート）
pub mod grpc_native;
// connect_bidi: Connect-RPC bidi（全 5 class、fetch full-duplex streams）
pub mod connect_bidi;
// web_transport: WebTransport H/3（全 5 class、requires_fallback=true）
pub mod web_transport;
// sse_paired: EventSource SSE（v1_alert / v1_event_feed / v1_live_snapshot）
pub mod sse_paired;
// paired_post_sse: POST↔SSE 半二重 emulation（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）
pub mod paired_post_sse;
// long_poll: fetch long-poll（v1_event_feed のみ）
pub mod long_poll;
// webhook: HMAC-SHA256 signed webhook（v1_alert / v1_event_feed / v1_live_snapshot）
pub mod webhook;
// messaging_bridge: Kafka producer（v1_event_feed / v1_live_snapshot / v1_bulk_upload）
pub mod messaging_bridge;

// AdapterManifest は各 adapter が自己宣言する capability 情報を宣言する。
// tier1 build script がこの情報を集約して capabilities.lock.yaml を生成する。
#[derive(Debug, Clone)]
pub struct AdapterManifest {
    // adapter_id: adapter の識別子（spec §adapter↔class supports 対応と一致）
    pub adapter_id: &'static str,
    // supports: サポートする conformance_class の一覧
    pub supports: &'static [&'static str],
    // requires_fallback: WebTransport 等で fallback が必要な場合 true
    pub requires_fallback: bool,
    // constraints: 物理的な制約（例: partition_key=session_id 必須）
    pub constraints: &'static str,
}
