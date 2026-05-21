// proto_bridge.go — k1s0 tier1 Library Go: proto layer（3-layer split の第 3 層）
// Bidi 適合仕様 3-layer split 規約（Y-bidi-library-split）に準拠する。
//
// ============================================================
// 3-layer split 規約
// ============================================================
// 本パッケージは以下 3 層のうち「proto layer」に該当する。
//
//   Layer 1: sdk（公開 API）
//     - tier2 / tier3 / client が消費する公開 API を提供する。
//     - KeyHandle / AuthContext / K1s0Logger 等の型を公開する。
//     - auth_context / keyhandle / observability / cache / db 等の各パッケージが担当する。
//
//   Layer 2: internal（server 内部 API）
//     - tier1 server 実装のみが使う内部 API を提供する。
//     - Go の package internal 規則で外部パッケージからのアクセスを禁止する。
//     - （将来） library/go/internal/ パッケージに実装する。
//
//   Layer 3: proto_bridge（Buf codegen 出力の薄い wrapper）
//     - buf generate 出力の protobuf 型 → Library 型への変換のみを担う。
//     - proto 型を公開 API シグネチャに露出しない（opaque ラップのみ）。
//     - 本パッケージ（proto_bridge/）が担当する。
//
// ============================================================
// 実装方針（stub）
// ============================================================
// 本ファイルは Y-bidi-library-split YELLOW 解消のための stub 実装。
// buf generate 出力型は src/tier1/schema/generated/go/ に配置予定。
// P7（crosscutting）フェーズで Buf codegen と接続して完全実装に移行する。

// パッケージ名: proto_bridge（tier1 Library の proto layer を担当する）
package proto_bridge

// BidiMessageProto は Buf codegen 出力 BidiMessage proto 型の薄い wrapper。
// proto 型を直接公開 API に露出せず、Library 型への変換のみを担う。
// 完全実装では generated.tier1.bidi.v1.BidiMessage を内包する。
// stub: proto 型の placeholder として生バイト列を使用する（Buf codegen 接続前の仮実装）
type BidiMessageProto struct {
    // rawBytes は wire format の proto バイト列を保持する（stub）
    rawBytes []byte
}

// NewBidiMessageProto は raw proto bytes から BidiMessageProto を構築する stub コンストラクタ。
// Buf codegen 接続後は protojson.Unmarshal() 等に置き換える。
// rawBytes: wire format の proto バイト列
func NewBidiMessageProto(rawBytes []byte) *BidiMessageProto {
    // rawBytes を BidiMessageProto にラップして返す（stub）
    return &BidiMessageProto{rawBytes: rawBytes}
}

// IntoRawBytes は BidiMessageProto から raw proto bytes を返す。
// Library 型への変換が不要な場合のエスケープハッチ（内部専用 / パッケージ外公開はしない）。
// 将来的には proto_bridge パッケージ内部にのみ公開する。
func (m *BidiMessageProto) IntoRawBytes() []byte {
    // rawBytes フィールドの値を返す（copy しない; 呼び出し元が所有権を持つ）
    return m.rawBytes
}

// ResumeTokenProto は Buf codegen 出力 ResumeToken proto 型の薄い wrapper。
// tls_disconnect / resume_after_disconnect scenario の resume_token を表現する。
// 完全実装では generated.tier1.bidi.v1.ResumeToken を内包する。
// stub: resume_token_hlc_valid assertion に対応する HLC フィールドを仮定義する
type ResumeTokenProto struct {
    // hlcMicros は HLC タイムスタンプ（マイクロ秒）を保持する（stub）
    // wall-clock ではなく HLC を必ず使用する（src/CLAUDE.md §wall-clock TTL 禁止に準拠）
    hlcMicros uint64
    // sessionID はセッション ID を保持する（stub）
    sessionID string
}

// NewResumeTokenProto は HLC タイムスタンプとセッション ID から ResumeTokenProto を構築する stub コンストラクタ。
// hlcMicros: HLC タイムスタンプ（マイクロ秒）
// sessionID: セッション ID 文字列
func NewResumeTokenProto(hlcMicros uint64, sessionID string) *ResumeTokenProto {
    // フィールドを初期化した ResumeTokenProto を返す（stub）
    return &ResumeTokenProto{hlcMicros: hlcMicros, sessionID: sessionID}
}

// HlcMicros は HLC タイムスタンプを返す（resume_token_hlc_valid assertion に使用する）。
func (r *ResumeTokenProto) HlcMicros() uint64 {
    // hlcMicros フィールドの値を返す
    return r.hlcMicros
}

// SessionID はセッション ID を返す（resume_after_disconnect assertion に使用する）。
func (r *ResumeTokenProto) SessionID() string {
    // sessionID フィールドの値を返す
    return r.sessionID
}
