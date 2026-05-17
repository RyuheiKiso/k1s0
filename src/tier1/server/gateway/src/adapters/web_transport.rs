// web_transport.rs — spec 01 Bidi §adapter web_transport
// 全 5 conformance_class をサポートする WebTransport H/3 adapter。
// requires_fallback=true: WebTransport 非対応 UA は connect_bidi / sse_paired にフォールバック。
// wtransport crate を使用して HTTP/3 ALPN over QUIC を実装する。

use super::AdapterManifest;

// MANIFEST は web_transport adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "web_transport",
    // web_transport は全 5 conformance_class をサポートする（v1 opt-in）
    supports: &[
        "v1_interactive",    // 双方向対話 WebTransport bidi stream
        "v1_alert",          // サーバー主導警報 WebTransport server stream
        "v1_event_feed",     // Domain Event 配信 WebTransport server stream
        "v1_live_snapshot",  // 最新値表示 WebTransport server stream
        "v1_bulk_upload",    // 大量データ投入 WebTransport client stream
    ],
    // WebTransport は Chromium 97+ のみ対応。非対応 UA は fallback が必須
    requires_fallback: true,
    // HTTP/3 QUIC が必要（UDP 443 を開放する必要がある）
    constraints: "http3=required, quic=required, ua_min=chromium97",
};

// WebTransportAdapter は WebTransport H/3 adapter。
pub struct WebTransportAdapter;

impl WebTransportAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }
}
