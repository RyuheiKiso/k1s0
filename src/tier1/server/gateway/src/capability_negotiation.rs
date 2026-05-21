// capability_negotiation.rs — spec 01 Bidi §Capability Negotiation
// 01_Bidi適合仕様.md §adapter↔class supports 対応 に基づく chosen_transport 選択アルゴリズムを実装する。
// capabilities.lock.yaml を起動時に 1 度だけ deserialize し、OnceLock で in-memory に保持する。
// hardcoded match は完全削除: supports の真は capabilities.lock.yaml が持つ。

// serde: CapabilitiesLock / CatalogEntry / NegotiationRequest / NegotiationResult の derive に使用する
use serde::{Deserialize, Serialize};
// std::collections::HashMap: adapter_id → supports リスト の索引に使用する
use std::collections::HashMap;
// std::sync::OnceLock: capabilities を起動時 1 回だけ初期化してプロセス全体で共有する
use std::sync::OnceLock;

// CAPABILITIES_LOCK_YAML_PATH_ENV: 環境変数名（未設定時はデフォルトパスを使う）
const CAPABILITIES_LOCK_YAML_PATH_ENV: &str = "K1S0_CAPABILITIES_LOCK_PATH";
// CAPABILITIES_LOCK_YAML_DEFAULT_PATH: capabilities.lock.yaml のデフォルト配置パス
// src/ root からの相対パス（プロセス起動ディレクトリを monorepo root と想定する）
const CAPABILITIES_LOCK_YAML_DEFAULT_PATH: &str =
    "src/tier1/lock/capabilities.lock.yaml";

// CapabilitiesLockCell: capabilities を OnceLock で保持する型エイリアス
// HashMap<adapter_id, Vec<conformance_class_id>>
static CAPABILITIES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

// CatalogCell は capabilities.lock.yaml の cells 配列の各要素を表す。
// フィールドは capabilities.lock.yaml の構造に 1:1 対応する。
#[derive(Debug, Deserialize)]
struct CatalogCell {
    // cell_id: "{conformance_class}__{adapter}" 形式の一意識別子
    #[allow(dead_code)]
    cell_id: String,
    // conformance_class: このセルの Bidi class（"v1_interactive" 等）
    conformance_class: String,
    // adapter: このセルの adapter 名（"grpc_native" 等）
    adapter: String,
    // status: "green" / "not_applicable"（"not_applicable" は supports に含めない）
    status: String,
    // assertion_run_result_id: 最終 assertion 実行 ID（起動時 verify で参照する）
    #[allow(dead_code)]
    assertion_run_result_id: String,
}

// CapabilitiesLock は capabilities.lock.yaml のトップレベル構造体を表す。
#[derive(Debug, Deserialize)]
struct CapabilitiesLock {
    // metadata: catalog_sot_hash / catalog_validation_status 等を含むメタデータブロック
    metadata: CapabilitiesMetadata,
    // cells: 全 40 cell（5 conformance_class × 8 adapter）の一覧
    cells: Vec<CatalogCell>,
}

// CapabilitiesMetadata は capabilities.lock.yaml の metadata ブロックを表す。
#[derive(Debug, Deserialize)]
struct CapabilitiesMetadata {
    // catalog_sot_hash: sha256:<hash> 形式の SOT hash（起動時に verify する）
    catalog_sot_hash: String,
    // catalog_validation_status: "passed" であることを起動時に確認する
    catalog_validation_status: String,
    // catalog_classes_path: classes.yaml のパス（参照用）
    #[allow(dead_code)]
    catalog_classes_path: String,
    // catalog_scenarios_path: scenarios.yaml のパス（参照用）
    #[allow(dead_code)]
    catalog_scenarios_path: String,
}

// load_capabilities_from_lock は capabilities.lock.yaml を読み込み、
// adapter_id → supports: Vec<conformance_class> の HashMap を返す。
// deserialize 失敗 / validation 失敗は panic!（fail-fast: 起動時エラーを即座に検知する）。
fn load_capabilities_from_lock() -> HashMap<String, Vec<String>> {
    // 環境変数 K1S0_CAPABILITIES_LOCK_PATH を優先し、未設定ならデフォルトパスを使用する
    let path = std::env::var(CAPABILITIES_LOCK_YAML_PATH_ENV)
        .unwrap_or_else(|_| CAPABILITIES_LOCK_YAML_DEFAULT_PATH.to_string());
    // capabilities.lock.yaml を UTF-8 テキストとして読み込む
    let yaml_text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        // ファイル読み込み失敗は起動不可とし、即座に panic する
        panic!(
            "capabilities.lock.yaml load failed: path={path} error={e}",
            path = path,
            e = e
        )
    });
    // YAML をデシリアライズして CapabilitiesLock 構造体に変換する
    let lock: CapabilitiesLock = serde_yaml::from_str(&yaml_text).unwrap_or_else(|e| {
        // YAML パース失敗は設計上あり得ない（CI が生成を保証する）ため即座に panic する
        panic!(
            "capabilities.lock.yaml parse failed: path={path} error={e}",
            path = path,
            e = e
        )
    });
    // catalog_validation_status が "passed" であることを起動時に verify する
    if lock.metadata.catalog_validation_status != "passed" {
        // validation_status が passed 以外は capabilities が不完全なため起動を拒否する
        panic!(
            "capabilities.lock.yaml catalog_validation_status is not 'passed': got={status}",
            status = lock.metadata.catalog_validation_status
        );
    }
    // catalog_sot_hash が "sha256:" プレフィックスを持つことを起動時に verify する
    // （完全な hash 照合は tools/generate_capabilities.py の責務、ここでは形式チェックのみ行う）
    if !lock.metadata.catalog_sot_hash.starts_with("sha256:") {
        // catalog_sot_hash の形式が不正な場合は起動を拒否する
        panic!(
            "capabilities.lock.yaml catalog_sot_hash has invalid format: got={hash}",
            hash = lock.metadata.catalog_sot_hash
        );
    }
    // cells を走査して adapter_id → supports: Vec<conformance_class> の HashMap を構築する
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    // 全 cell を処理する（40 cell = 5 class × 8 adapter）
    for cell in &lock.cells {
        // status が "green" の cell のみ supports に含める（"not_applicable" は除外する）
        if cell.status == "green" {
            // adapter_id をキーに conformance_class を追加する
            map.entry(cell.adapter.clone())
                .or_default()
                .push(cell.conformance_class.clone());
        }
    }
    // 構築した HashMap を返す（OnceLock に格納される）
    map
}

// get_capabilities は起動時に 1 度だけ load_capabilities_from_lock を実行し、
// 以降はキャッシュ済みの HashMap への参照を返す。
fn get_capabilities() -> &'static HashMap<String, Vec<String>> {
    // OnceLock::get_or_init で初回呼び出し時のみ load_capabilities_from_lock を実行する
    CAPABILITIES.get_or_init(load_capabilities_from_lock)
}

// get_supports_for_adapter は adapter 名が supports する conformance_class 一覧を返す。
// capabilities.lock.yaml から起動時に読み込んだ値を返す（hardcoded match は使用しない）。
fn get_supports_for_adapter(adapter_name: &str) -> &'static [String] {
    // get_capabilities から OnceLock キャッシュを取得する
    let caps = get_capabilities();
    // adapter_name に対応する supports リストを返す（未知の adapter は空スライスを返す）
    caps.get(adapter_name)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

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

// adapter_from_str は文字列から AdapterKind に変換する
fn adapter_from_str(s: &str) -> Option<AdapterKind> {
    // spec §adapter↔class supports 対応 の adapter 名と 1:1 対応する
    match s {
        // grpc_native: gRPC native streaming adapter
        "grpc_native" => Some(AdapterKind::GrpcNative),
        // connect_bidi: Connect-RPC bidi streaming adapter
        "connect_bidi" => Some(AdapterKind::ConnectBidi),
        // web_transport: WebTransport H/3 adapter（requires_fallback=true）
        "web_transport" => Some(AdapterKind::WebTransport),
        // sse_paired: EventSource SSE adapter
        "sse_paired" => Some(AdapterKind::SsePaired),
        // paired_post_sse: POST↔SSE pair adapter
        "paired_post_sse" => Some(AdapterKind::PairedPostSse),
        // long_poll: fetch long-poll adapter
        "long_poll" => Some(AdapterKind::LongPoll),
        // webhook: HMAC-SHA256 signed webhook adapter
        "webhook" => Some(AdapterKind::Webhook),
        // messaging_bridge: Kafka messaging bridge adapter
        "messaging_bridge" => Some(AdapterKind::MessagingBridge),
        // 未知の adapter 名は None を返す（negotiate で skip される）
        _ => None,
    }
}

// negotiate は NegotiationRequest から最適な adapter を選択して返す。
// spec §Capability Negotiation アルゴリズム（129-134 行）の実装:
//   1. preferred_adapters を priority 順に走査する
//   2. conformance_class が capabilities.lock.yaml の supports リストに含まれる最初の adapter を選択する
//   3. preferred_adapters が空か全て不適合 → grpc_native にフォールバックする
pub fn negotiate(req: &NegotiationRequest) -> NegotiationResult {
    // preferred_adapters の優先順に capabilities.lock.yaml から読み込んだ supports を確認する
    for adapter_name in &req.preferred_adapters {
        // 文字列を AdapterKind に変換する（未知の adapter 名はスキップ）
        if let Some(adapter) = adapter_from_str(adapter_name) {
            // capabilities.lock.yaml から adapter の supports リストを取得する
            let supports = get_supports_for_adapter(adapter_name);
            // adapter が conformance_class をサポートするか確認する
            if supports.iter().any(|s| s == &req.conformance_class) {
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
    // grpc_native は全 5 class をサポートするため capabilities.lock.yaml で必ず選択可能
    NegotiationResult {
        // grpc_native を fallback として選択する
        chosen_adapter: AdapterKind::GrpcNative,
        // fallback 先の adapter 名を設定する
        chosen_adapter_name: "grpc_native".to_string(),
        // preferred_adapters が指定されていた場合は fallback_used=true とする
        fallback_used: !req.preferred_adapters.is_empty(),
        // grpc_native は requires_fallback 不要
        requires_fallback: false,
    }
}
