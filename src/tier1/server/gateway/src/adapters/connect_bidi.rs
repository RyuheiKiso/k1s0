// connect_bidi.rs — spec 01 Bidi §adapter connect_bidi
// 全 5 conformance_class をサポートする Connect-RPC bidi adapter。
// fetch full-duplex streams を使用し、UA subclass 別に adapter を選択する。
// 13_dotnet8_connect_inhouse.md の Connect-RPC inhouse 実装と連携する。

use super::AdapterManifest;

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
