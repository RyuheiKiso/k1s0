// paired_post_sse.rs — spec 01 Bidi §adapter paired_post_sse
// POST↔SSE pair adapter: 半二重 emulation（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）。
// v1_bulk_upload は半二重 emulation では担えないため not_applicable。
// 12_UA_aware_adapter.md の K1s0UaAwareAdapter と連携して UA 判定を行う。
// 13_dotnet8_connect_inhouse.md の .NET Framework 対応に使用する。

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
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }
}
