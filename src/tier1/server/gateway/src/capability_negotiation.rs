// capability_negotiation.rs — spec 01 Bidi §Capability Negotiation
// 01_Bidi適合仕様.md §adapter↔class supports 対応 に基づく chosen_transport 選択アルゴリズムを実装する。
// 各 adapter の supports 集合と UA 情報から最適な transport adapter を決定する。

use serde::{Deserialize, Serialize};

// AdapterKind は 8 transport adapter を宣言する（spec §adapter↔class supports 対応）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    // gRPC native streaming（全 5 class をサポート）
    GrpcNative,
    // Connect-RPC bidi streaming（全 5 class、fetch full-duplex streams）
    ConnectBidi,
    // WebTransport H/3（全 5 class、requires_fallback=true）
    WebTransport,
    // SSE paired（v1_alert / v1_event_feed / v1_live_snapshot のみ）
    SsePaired,
    // POST↔SSE pair（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）
    PairedPostSse,
    // fetch long-poll（v1_event_feed のみ）
    LongPoll,
    // HMAC-SHA256 webhook（v1_alert / v1_event_feed / v1_live_snapshot）
    Webhook,
    // Kafka messaging bridge（v1_event_feed / v1_live_snapshot / v1_bulk_upload）
    MessagingBridge,
}

// NegotiationRequest は client から送られる adapter 選択リクエストを宣言する。
#[derive(Debug, Clone, Deserialize)]
pub struct NegotiationRequest {
    // conformance_class: 要求する Bidi class
    pub conformance_class: String,
    // preferred_adapters: client が希望する adapter の優先順位リスト
    pub preferred_adapters: Vec<String>,
    // ua_hint: User-Agent ベースのヒント（client 層が UA 判定して送信する）
    pub ua_hint: Option<String>,
}

// NegotiationResult は選択された transport adapter を返す。
#[derive(Debug, Clone, Serialize)]
pub struct NegotiationResult {
    // chosen_adapter: 選択された transport adapter
    pub chosen_adapter: AdapterKind,
    // chosen_adapter_name: adapter の文字列名（spec §adapter↔class supports 対応と一致）
    pub chosen_adapter_name: String,
    // fallback_used: requires_fallback=true の adapter を使用した場合 true
    pub fallback_used: bool,
    // requires_fallback: WebTransport など fallback が必要な場合 true
    pub requires_fallback: bool,
}

// supports_matrix は adapter の supports 集合を返す（spec §adapter↔class supports 対応）
fn supports_matrix(adapter: &AdapterKind) -> &'static [&'static str] {
    // 各 adapter が supports する conformance_class 一覧を返す
    match adapter {
        // grpc_native は全 5 class をサポートする
        AdapterKind::GrpcNative => &[
            "v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"
        ],
        // connect_bidi は全 5 class をサポートする（fetch full-duplex）
        AdapterKind::ConnectBidi => &[
            "v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"
        ],
        // web_transport は全 5 class をサポートするが requires_fallback=true
        AdapterKind::WebTransport => &[
            "v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"
        ],
        // sse_paired は client→server bulk（v1_bulk_upload）を構造的にサポートできない
        AdapterKind::SsePaired => &["v1_alert", "v1_event_feed", "v1_live_snapshot"],
        // paired_post_sse は半二重 emulation のため v1_bulk_upload は不向き
        AdapterKind::PairedPostSse => &[
            "v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot"
        ],
        // long_poll は v1_event_feed のみサポートする（低 lag 要件を満たせない）
        AdapterKind::LongPoll => &["v1_event_feed"],
        // webhook は v1_interactive（双方向）と v1_bulk_upload（client→server）をサポートできない
        AdapterKind::Webhook => &["v1_alert", "v1_event_feed", "v1_live_snapshot"],
        // messaging_bridge は partition_key=session_id 制約で v1_interactive / v1_alert をサポートしない
        AdapterKind::MessagingBridge => &[
            "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"
        ],
    }
}

// adapter_from_str は文字列から AdapterKind に変換する
fn adapter_from_str(s: &str) -> Option<AdapterKind> {
    // spec §adapter↔class supports 対応 の adapter 名と 1:1 対応する
    match s {
        "grpc_native" => Some(AdapterKind::GrpcNative),
        "connect_bidi" => Some(AdapterKind::ConnectBidi),
        "web_transport" => Some(AdapterKind::WebTransport),
        "sse_paired" => Some(AdapterKind::SsePaired),
        "paired_post_sse" => Some(AdapterKind::PairedPostSse),
        "long_poll" => Some(AdapterKind::LongPoll),
        "webhook" => Some(AdapterKind::Webhook),
        "messaging_bridge" => Some(AdapterKind::MessagingBridge),
        _ => None,
    }
}

// negotiate は NegotiationRequest から最適な adapter を選択して返す。
// spec §Capability Negotiation アルゴリズム（129-134 行）の実装:
//   1. preferred_adapters を priority 順に走査する
//   2. conformance_class が supports_matrix に含まれる最初の adapter を選択する
//   3. preferred_adapters が空か全て不適合 → grpc_native にフォールバックする
pub fn negotiate(req: &NegotiationRequest) -> NegotiationResult {
    // preferred_adapters の優先順に supports を確認する
    for adapter_name in &req.preferred_adapters {
        // 文字列を AdapterKind に変換する（未知の adapter 名はスキップ）
        if let Some(adapter) = adapter_from_str(adapter_name) {
            // adapter が conformance_class をサポートするか確認する
            if supports_matrix(&adapter).contains(&req.conformance_class.as_str()) {
                // requires_fallback フラグを WebTransport のみ true にする
                let requires_fallback = adapter == AdapterKind::WebTransport;
                // 選択された adapter を返す
                return NegotiationResult {
                    chosen_adapter_name: adapter_name.clone(),
                    fallback_used: false,
                    requires_fallback,
                    chosen_adapter: adapter,
                };
            }
        }
    }
    // preferred_adapters が全て不適合 → grpc_native にフォールバックする
    // grpc_native は全 5 class をサポートするため必ず選択可能
    NegotiationResult {
        chosen_adapter: AdapterKind::GrpcNative,
        chosen_adapter_name: "grpc_native".to_string(),
        fallback_used: !req.preferred_adapters.is_empty(),
        requires_fallback: false,
    }
}
