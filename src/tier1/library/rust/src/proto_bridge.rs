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
//   Layer 3: proto_bridge（proto 型 → Library 型への変換を担う）
//     - gateway が受け取った raw gRPC フレームを BidiEnvelope に変換する。
//     - proto 型を公開 API シグネチャに露出しない（opaque ラップのみ）。
//     - 本モジュール（proto_bridge.rs）が担当する。
//
// ============================================================
// 実装方針
// ============================================================
// tier1 gateway は axum で gRPC フレーム（HTTP/2 DATA frame）を直接処理する。
// Rust codegen（prost/tonic）を使わず、serde_json による JSON トランスコード経路と
// 手書き proto3 デコード経路の両方を提供する。
// proto SoT: src/tier1/schema/bidi/tier1/bidi/v1/service.proto §BidiEnvelope

// serde: BidiEnvelope / ResumeTokenProto の JSON シリアライズ/デシリアライズに使用する
use serde::{Deserialize, Serialize};

// proto_bridge モジュールは pub(crate) スコープで tier1 server 内部からのみ使用する。
// 公開 API（sdk layer）への proto 型露出は禁止する。

/// BidiEnvelope は service.proto §BidiEnvelope の Rust 表現。
/// gateway が受け取った JSON トランスコード済みボディを serde_json でデシリアライズする。
/// proto SoT: src/tier1/schema/bidi/tier1/bidi/v1/service.proto
// BidiEnvelope 構造体: proto3 BidiEnvelope の Rust mapping（serde_json 経路）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BidiEnvelope {
    // seq: stream 内のメッセージシーケンス番号（単調増加、0 起算）
    // proto field: uint64 seq = 1
    #[serde(default)]
    pub(crate) seq: u64,
    // payload: アプリケーションペイロード（base64 エンコードされた opaque bytes）
    // proto field: bytes payload = 2（JSON では base64 文字列として転送される）
    #[serde(default)]
    pub(crate) payload: String,
    // trace_id: W3C traceparent trace-id 部分（16 byte hex 文字列）
    // proto field: string trace_id = 3
    #[serde(default)]
    pub(crate) trace_id: String,
    // span_id: W3C traceparent span-id 部分（8 byte hex 文字列）
    // proto field: string span_id = 4
    #[serde(default)]
    pub(crate) span_id: String,
    // actor_id: 操作主体 canonical subject_id（AuthContext.subject_id と一致させる）
    // proto field: string actor_id = 5
    #[serde(default)]
    pub(crate) actor_id: String,
    // aggregate_qualified_name: 操作対象の集約修飾名（"<bc>.<Aggregate>/<id>" 形式）
    // proto field: string aggregate_qualified_name = 6
    #[serde(default)]
    pub(crate) aggregate_qualified_name: String,
    // resume_token: v1_interactive / v1_alert / v1_event_feed における再接続トークン
    // proto field: string resume_token = 7
    #[serde(default)]
    pub(crate) resume_token: String,
    // conformance_class: このフレームが属する Bidi conformance class（classes.yaml の id と一致）
    // proto field: string conformance_class = 8
    #[serde(default)]
    pub(crate) conformance_class: String,
}

impl BidiEnvelope {
    /// from_json は JSON バイト列（envoy transcoding 経路）から BidiEnvelope を構築する。
    // from_json: serde_json でデシリアライズする
    pub(crate) fn from_json(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        // bytes を UTF-8 文字列として serde_json でパースする
        serde_json::from_slice(bytes)
    }

    /// to_json は BidiEnvelope を JSON バイト列にシリアライズする。
    // to_json: serde_json でシリアライズする
    pub(crate) fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        // self を JSON バイト列に変換する
        serde_json::to_vec(self)
    }

    /// from_raw_proto は gRPC LENGTH-PREFIXED wire format バイト列から BidiEnvelope を構築する。
    /// gRPC フレーム: 1B compression flag + 4B big-endian length + N bytes proto3 payload
    // from_raw_proto: gRPC framing を剥がして JSON ボディとして解釈する（JSON トランスコード非適用経路）
    pub(crate) fn from_raw_proto(frame: &[u8]) -> Result<Self, ProtoDecodeError> {
        // gRPC フレームは最低 5 バイト必要（compression flag + 4B length）
        if frame.len() < 5 {
            // フレームが短すぎる場合はエラーを返す
            return Err(ProtoDecodeError::InvalidFrame);
        }
        // 5 バイトのヘッダーをスキップして proto3 ボディを取り出す
        let body = &frame[5..];
        // proto3 wire format を手動デコードする（tag-value パース）
        decode_bidi_envelope_proto3(body)
    }

    /// into_raw_bytes は BidiEnvelope.payload の raw bytes を返す。
    /// Library 型（tier2 BidiMessage）への変換に使用する。
    // into_raw_bytes: payload を base64 デコードして raw bytes を返す
    pub(crate) fn payload_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        // base64 デコードして raw bytes を返す（gateway の JSON 経路は base64 エンコード）
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(&self.payload)
    }
}

/// ProtoDecodeError は proto3 デコード中に発生するエラーを宣言する。
// ProtoDecodeError 列挙型: デコードエラー分類
#[derive(Debug)]
pub(crate) enum ProtoDecodeError {
    // gRPC フレームが不正（5 バイト未満）
    InvalidFrame,
    // proto3 wire format が不正（不正な varint / tag / field type）
    MalformedProto,
    // UTF-8 デコードエラー（string フィールド）
    InvalidUtf8,
}

/// decode_bidi_envelope_proto3 は proto3 wire format の raw bytes から BidiEnvelope を構築する。
/// proto3 wire format: tag = (field_number << 3) | wire_type
/// wire_type: 0=varint, 2=length-delimited
// decode_bidi_envelope_proto3: proto3 TLV ループで BidiEnvelope を構築する
fn decode_bidi_envelope_proto3(bytes: &[u8]) -> Result<BidiEnvelope, ProtoDecodeError> {
    // デフォルト値で BidiEnvelope を初期化する
    let mut env = BidiEnvelope {
        seq: 0,
        payload: String::new(),
        trace_id: String::new(),
        span_id: String::new(),
        actor_id: String::new(),
        aggregate_qualified_name: String::new(),
        resume_token: String::new(),
        conformance_class: String::new(),
    };
    // bytes 全体を消費するまでループする
    let mut pos = 0usize;
    while pos < bytes.len() {
        // tag を varint デコードする（field_number + wire_type）
        let (tag, tag_len) = decode_varint(bytes, pos)?;
        // tag_len バイト進める
        pos += tag_len;
        // wire_type は tag の下位 3 bit
        let wire_type = tag & 0x07;
        // field_number は tag の上位ビット（wire_type を除く）
        let field_number = (tag >> 3) as u32;
        match wire_type {
            // varint (wire_type = 0): uint64 フィールド
            0 => {
                // varint 値をデコードする
                let (val, len) = decode_varint(bytes, pos)?;
                // pos を進める
                pos += len;
                // field 1: seq
                if field_number == 1 {
                    // seq フィールドに設定する
                    env.seq = val;
                }
                // 他の varint フィールドは BidiEnvelope に存在しない（スキップ）
            }
            // length-delimited (wire_type = 2): bytes / string フィールド
            2 => {
                // 長さ prefix をデコードする
                let (len, len_field_size) = decode_varint(bytes, pos)?;
                // pos を len_field_size 進める
                pos += len_field_size;
                // len バイトのデータを取り出す
                let end = pos + len as usize;
                if end > bytes.len() {
                    // データが不足している場合はエラーを返す
                    return Err(ProtoDecodeError::MalformedProto);
                }
                // データスライスを取り出す
                let data = &bytes[pos..end];
                // pos を end に進める
                pos = end;
                // field number に応じてフィールドに設定する
                match field_number {
                    // payload: bytes フィールド（base64 エンコードではなく raw bytes → base64 に変換する）
                    2 => {
                        use base64::Engine;
                        // raw bytes を base64 エンコードして格納する（to_json との整合性を保つ）
                        env.payload = base64::engine::general_purpose::STANDARD.encode(data);
                    }
                    // trace_id: string フィールド
                    3 => {
                        // UTF-8 デコードして格納する
                        env.trace_id = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // span_id: string フィールド
                    4 => {
                        // UTF-8 デコードして格納する
                        env.span_id = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // actor_id: string フィールド
                    5 => {
                        // UTF-8 デコードして格納する
                        env.actor_id = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // aggregate_qualified_name: string フィールド
                    6 => {
                        // UTF-8 デコードして格納する
                        env.aggregate_qualified_name = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // resume_token: string フィールド
                    7 => {
                        // UTF-8 デコードして格納する
                        env.resume_token = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // conformance_class: string フィールド
                    8 => {
                        // UTF-8 デコードして格納する
                        env.conformance_class = std::str::from_utf8(data)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_owned();
                    }
                    // 未知のフィールドはスキップする（proto3 互換性）
                    _ => {}
                }
            }
            // wire_type 1: 64-bit (double / fixed64) — BidiEnvelope には存在しないのでスキップ
            1 => {
                // 8 バイトスキップする
                pos += 8;
            }
            // wire_type 5: 32-bit (float / fixed32) — BidiEnvelope には存在しないのでスキップ
            5 => {
                // 4 バイトスキップする
                pos += 4;
            }
            // 未知の wire_type はエラー
            _ => {
                // 不正な wire_type はデコードエラーとして扱う
                return Err(ProtoDecodeError::MalformedProto);
            }
        }
    }
    // デコード結果を返す
    Ok(env)
}

/// decode_varint は proto3 LEB128 varint を pos バイト目からデコードする。
/// 戻り値: (値, 消費バイト数)
// decode_varint: LEB128 可変長整数デコーダ
fn decode_varint(bytes: &[u8], pos: usize) -> Result<(u64, usize), ProtoDecodeError> {
    // 結果値と消費バイト数を初期化する
    let mut result: u64 = 0;
    // 消費バイト数
    let mut consumed = 0usize;
    // 最大 10 バイト（64bit varint の最大長）まで処理する
    for i in 0..10 {
        // pos + i バイト目を取得する
        let b = *bytes.get(pos + i).ok_or(ProtoDecodeError::MalformedProto)?;
        // 下位 7 bit を result に格納する（7 bit × (消費バイト数) だけシフト）
        result |= ((b & 0x7f) as u64) << (7 * i);
        // 消費バイト数をインクリメントする
        consumed += 1;
        // MSB が 0 であれば varint 終了
        if b & 0x80 == 0 {
            // デコード完了
            return Ok((result, consumed));
        }
    }
    // 10 バイトを超えた場合は不正な varint
    Err(ProtoDecodeError::MalformedProto)
}

/// ResumeTokenProto は resume_token の HLC + session_id を表す。
/// tls_disconnect / resume_after_disconnect scenario で使用する。
/// proto SoT: service.proto §BidiEnvelope.resume_token フィールドのセマンティクス
// ResumeTokenProto 構造体: resume_token の構造化表現
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResumeTokenProto {
    // HLC タイムスタンプ（マイクロ秒）を保持する
    // wall-clock ではなく HLC を必ず使用する（src/CLAUDE.md §wall-clock TTL 禁止に準拠）
    pub(crate) hlc_micros: u64,
    // セッション ID を保持する（UUID v4 形式）
    pub(crate) session_id: String,
}

impl ResumeTokenProto {
    /// new は HLC タイムスタンプとセッション ID から ResumeTokenProto を構築する。
    // new コンストラクタ: hlc_micros と session_id を受け取って初期化する
    pub(crate) fn new(hlc_micros: u64, session_id: String) -> Self {
        // フィールドを初期化する
        Self { hlc_micros, session_id }
    }

    /// from_token_string は "hlc:{hlc_micros}:sid:{session_id}" 形式の文字列を解析する。
    // from_token_string: resume_token 文字列から構造化データを取り出す
    pub(crate) fn from_token_string(token: &str) -> Option<Self> {
        // "hlc:" プレフィックスを確認する
        let rest = token.strip_prefix("hlc:")?;
        // ":sid:" で分割する
        let mut parts = rest.splitn(2, ":sid:");
        // hlc_micros を取り出す
        let hlc_str = parts.next()?;
        // session_id を取り出す
        let sid = parts.next()?;
        // hlc_micros を u64 にパースする
        let hlc_micros = hlc_str.parse::<u64>().ok()?;
        // ResumeTokenProto を構築して返す
        Some(Self { hlc_micros, session_id: sid.to_owned() })
    }

    /// to_token_string は "hlc:{hlc_micros}:sid:{session_id}" 形式の文字列に変換する。
    // to_token_string: resume_token 文字列形式にシリアライズする
    pub(crate) fn to_token_string(&self) -> String {
        // "hlc:" + hlc_micros + ":sid:" + session_id を連結して返す
        format!("hlc:{}:sid:{}", self.hlc_micros, self.session_id)
    }

    /// hlc_micros は HLC タイムスタンプを返す（resume_token_hlc_valid assertion に使用する）。
    // hlc_micros getter: HLC タイムスタンプを返す
    pub(crate) fn hlc_micros(&self) -> u64 {
        // hlc_micros フィールドの値を返す
        self.hlc_micros
    }

    /// session_id はセッション ID を返す（resume_after_disconnect assertion に使用する）。
    // session_id getter: セッション ID を返す
    pub(crate) fn session_id(&self) -> &str {
        // session_id フィールドの参照を返す
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    // super モジュールのシンボルを全てインポートする
    use super::*;

    // test_bidi_envelope_from_json は JSON デシリアライズを検証するテスト
    #[test]
    fn test_bidi_envelope_from_json() {
        // テスト用 JSON を構築する
        let json = br#"{"seq":1,"trace_id":"abc","actor_id":"user-001","conformance_class":"c1_bidirectional_full"}"#;
        // JSON から BidiEnvelope をデシリアライズする
        let env = BidiEnvelope::from_json(json).unwrap();
        // seq フィールドを確認する
        assert_eq!(env.seq, 1);
        // trace_id フィールドを確認する
        assert_eq!(env.trace_id, "abc");
        // conformance_class フィールドを確認する
        assert_eq!(env.conformance_class, "c1_bidirectional_full");
    }

    // test_bidi_envelope_round_trip は JSON の往復変換を検証するテスト
    #[test]
    fn test_bidi_envelope_round_trip() {
        // テスト用 BidiEnvelope を構築する
        let env = BidiEnvelope {
            seq: 42,
            payload: "aGVsbG8=".to_owned(),
            trace_id: "trace-001".to_owned(),
            span_id: "span-001".to_owned(),
            actor_id: "user-001".to_owned(),
            aggregate_qualified_name: "order.Order/uuid-001".to_owned(),
            resume_token: "hlc:123456:sid:session-001".to_owned(),
            conformance_class: "c1_bidirectional_full".to_owned(),
        };
        // JSON に変換する
        let json = env.to_json().unwrap();
        // JSON から再構築する
        let env2 = BidiEnvelope::from_json(&json).unwrap();
        // seq が一致することを確認する
        assert_eq!(env2.seq, 42);
        // resume_token が一致することを確認する
        assert_eq!(env2.resume_token, env.resume_token);
    }

    // test_resume_token_round_trip は resume_token 文字列変換を検証するテスト
    #[test]
    fn test_resume_token_round_trip() {
        // ResumeTokenProto を構築する
        let token = ResumeTokenProto::new(123456789, "session-001".to_owned());
        // to_token_string でシリアライズする
        let s = token.to_token_string();
        // from_token_string でデシリアライズする
        let recovered = ResumeTokenProto::from_token_string(&s).unwrap();
        // hlc_micros が一致することを確認する
        assert_eq!(recovered.hlc_micros(), 123456789);
        // session_id が一致することを確認する
        assert_eq!(recovered.session_id(), "session-001");
    }

    // test_decode_varint は LEB128 varint デコーダを検証するテスト
    #[test]
    fn test_decode_varint() {
        // 単一バイト varint (値 1) のデコードを検証する
        let (val, len) = decode_varint(&[0x01], 0).unwrap();
        assert_eq!(val, 1);
        assert_eq!(len, 1);
        // 2 バイト varint (値 128) のデコードを検証する（0x80 0x01 = 128）
        let (val2, len2) = decode_varint(&[0x80, 0x01], 0).unwrap();
        assert_eq!(val2, 128);
        assert_eq!(len2, 2);
    }
}
