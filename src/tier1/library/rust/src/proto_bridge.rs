// proto_bridge.rs — k1s0 tier1 Library Rust: proto layer（3-layer split の第 3 層）
// Bidi 適合仕様 3-layer split 規約（Y-bidi-library-split）に準拠する。
//
// ============================================================
// 3-layer split 規約
// ============================================================
// 本モジュールは以下 3 層のうち「proto layer」に該当する。
//
//   Layer 1: sdk（公開 API）
//     - tier2 / tier3 / client が消費する公開 API を提供する。
//     - KeyHandle / AuthContext / K1s0Logger 等の型を公開する。
//     - 本クレートの crate::core / crate::backend / crate::frontend が担当する。
//
//   Layer 2: internal（server 内部 API）
//     - tier1 server 実装のみが使う内部 API を提供する。
//     - 公開 API に含めない（pub(crate) スコープに限定する）。
//     - 本クレートでは `#[doc(hidden)]` + `pub(crate)` で区別する。
//
//   Layer 3: proto_bridge（Buf codegen 出力の薄い wrapper）
//     - `buf generate` 出力の protobuf 型 → Library 型への変換のみを担う。
//     - proto 型を公開 API シグネチャに露出しない（opaque ラップのみ）。
//     - 本モジュール（proto_bridge.rs）が担当する。
//
// ============================================================
// 実装方針（stub）
// ============================================================
// 本ファイルは Y-bidi-library-split YELLOW 解消のための stub 実装。
// buf generate 出力型は `src/tier1/schema/generated/rust/` に配置予定。
// P7（crosscutting）フェーズで Buf codegen と接続して完全実装に移行する。

// proto_bridge モジュールは pub(crate) スコープで tier1 server 内部からのみ使用する。
// 公開 API（sdk layer）への proto 型露出は禁止する。

/// BidiMessageProto は Buf codegen 出力 BidiMessage proto 型の薄い wrapper。
/// proto 型を直接公開 API に露出せず、Library 型への変換のみを担う。
/// 完全実装では `generated::tier1::bidi::v1::BidiMessage` を内包する。
// stub: proto 型の placeholder として u8 bytes を使用する（Buf codegen 接続前の仮実装）
#[derive(Debug, Clone)]
pub(crate) struct BidiMessageProto {
    // 生の proto バイト列（wire format）を保持する（stub）
    raw_bytes: Vec<u8>,
}

impl BidiMessageProto {
    /// new は raw proto bytes から BidiMessageProto を構築する stub コンストラクタ。
    /// Buf codegen 接続後は prost::Message::decode() 等に置き換える。
    // stub コンストラクタ: raw bytes をそのまま保持する
    pub(crate) fn new(raw_bytes: Vec<u8>) -> Self {
        // raw_bytes を BidiMessageProto にラップする（stub）
        Self { raw_bytes }
    }

    /// into_raw_bytes は BidiMessageProto から raw proto bytes を返す。
    /// Library 型への変換が不要な場合のエスケープハッチ（内部専用）。
    // into_raw_bytes: 保持している raw bytes を消費して返す
    pub(crate) fn into_raw_bytes(self) -> Vec<u8> {
        // raw_bytes フィールドを移動して返す
        self.raw_bytes
    }
}

/// ResumeTokenProto は Buf codegen 出力 ResumeToken proto 型の薄い wrapper。
/// tls_disconnect / resume_after_disconnect scenario の resume_token を表現する。
/// 完全実装では `generated::tier1::bidi::v1::ResumeToken` を内包する。
// stub: resume_token_hlc_valid assertion に対応する HLC フィールドを仮定義する
#[derive(Debug, Clone)]
pub(crate) struct ResumeTokenProto {
    // HLC タイムスタンプ（マイクロ秒）を保持する（stub）
    // wall-clock ではなく HLC を必ず使用する（src/CLAUDE.md §wall-clock TTL 禁止に準拠）
    hlc_micros: u64,
    // セッション ID を保持する（stub）
    session_id: String,
}

impl ResumeTokenProto {
    /// new は HLC タイムスタンプとセッション ID から ResumeTokenProto を構築する stub コンストラクタ。
    // stub コンストラクタ: hlc_micros と session_id を受け取って初期化する
    pub(crate) fn new(hlc_micros: u64, session_id: String) -> Self {
        // フィールドを初期化する
        Self { hlc_micros, session_id }
    }

    /// hlc_micros は HLC タイムスタンプを返す（resume_token_hlc_valid assertion に使用する）。
    // hlc_micros: HLC タイムスタンプを返す getter
    pub(crate) fn hlc_micros(&self) -> u64 {
        // hlc_micros フィールドの値を返す
        self.hlc_micros
    }

    /// session_id はセッション ID を返す（resume_after_disconnect assertion に使用する）。
    // session_id: セッション ID を返す getter（&str を返すことで所有権を保持する）
    pub(crate) fn session_id(&self) -> &str {
        // session_id フィールドの参照を返す
        &self.session_id
    }
}
