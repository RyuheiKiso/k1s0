// 本ファイルは PII 仮名化（pseudonymize）の純関数実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-PII-002（決定論的 HMAC-SHA256、salt 別空間、不可逆）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   field_type / value / salt から決定論的な仮名 ID を生成する。
//   出力は URL-safe base64（padding なし）。同一 salt + 同一 (field_type, value)
//   で同一出力。salt が変わると出力空間も変わる（テナント横断結合を防止）。
//
// 設計上の決定:
//   - HMAC 鍵は salt（呼出側が OpenBao 等で管理する想定）。
//   - 入力は `<field_type>:<value>` の連結。field_type を prefix に含めることで、
//     同一 value の異種別衝突（"foo" を NAME と EMAIL の両方で使った時）を防ぐ。
//   - 出力は SHA-256 の 32 byte を URL-safe base64（"-_"、padding なし）で表現。
//     base64 表記長は 43 文字、URL / ファイル名にそのまま使える。

// HMAC<SHA-256> 実装。
use hmac::{Hmac, Mac};
// SHA-256 ダイジェスト。
use sha2::Sha256;
// URL-safe base64 エンコード（padding なし）。
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;

/// HMAC-SHA-256 alias（hmac crate の慣用）。
type HmacSha256 = Hmac<Sha256>;

/// pseudonymize エラー型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudonymizeError {
    /// salt が空文字（仮名空間の分離が機能しないため拒否）。
    EmptySalt,
    /// value が空文字。
    EmptyValue,
    /// field_type が空文字。
    EmptyFieldType,
}

impl std::fmt::Display for PseudonymizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PseudonymizeError::EmptySalt => write!(f, "salt must not be empty"),
            PseudonymizeError::EmptyValue => write!(f, "value must not be empty"),
            PseudonymizeError::EmptyFieldType => write!(f, "field_type must not be empty"),
        }
    }
}

impl std::error::Error for PseudonymizeError {}

/// 仮名化を実行する。返却は URL-safe base64（padding なし）。
///
/// 引数:
///   - field_type: 仮名空間の種別（NAME / EMAIL / PHONE 等）
///   - value: 仮名化対象の生値
///   - salt: HMAC 鍵（OpenBao 管理の任意 byte 列を文字列で受け取る）
pub fn pseudonymize(
    field_type: &str,
    value: &str,
    salt: &str,
) -> Result<String, PseudonymizeError> {
    if salt.is_empty() {
        return Err(PseudonymizeError::EmptySalt);
    }
    if value.is_empty() {
        return Err(PseudonymizeError::EmptyValue);
    }
    if field_type.is_empty() {
        return Err(PseudonymizeError::EmptyFieldType);
    }
    // HMAC は鍵長任意可。salt の bytes をそのまま鍵にする。
    let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
        .expect("HMAC accepts any key length");
    // field_type を prefix として混入し、種別越境衝突を防ぐ。
    mac.update(field_type.as_bytes());
    // 区切り（field_type 内に ':' が含まれた場合の歪曲を避けるため固定 1 byte）。
    mac.update(b":");
    mac.update(value.as_bytes());
    let digest = mac.finalize().into_bytes();
    Ok(URL_SAFE_NO_PAD.encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同一入力 + 同一 salt は決定論的に同一出力。
    #[test]
    fn deterministic_same_inputs_produce_same_output() {
        let a = pseudonymize("EMAIL", "alice@example.com", "salt-A").unwrap();
        let b = pseudonymize("EMAIL", "alice@example.com", "salt-A").unwrap();
        assert_eq!(a, b);
    }

    /// 異なる salt は異なる出力（仮名空間分離）。
    #[test]
    fn different_salts_produce_different_output() {
        let a = pseudonymize("EMAIL", "alice@example.com", "salt-A").unwrap();
        let b = pseudonymize("EMAIL", "alice@example.com", "salt-B").unwrap();
        assert_ne!(a, b);
    }

    /// 同一 value でも種別が違えば出力空間が違う（NAME と EMAIL の衝突回避）。
    #[test]
    fn different_field_types_produce_different_output() {
        let n = pseudonymize("NAME", "alice", "salt-A").unwrap();
        let e = pseudonymize("EMAIL", "alice", "salt-A").unwrap();
        assert_ne!(n, e);
    }

    /// 出力は URL-safe base64（padding なし）の 43 文字（SHA-256 32 byte）。
    #[test]
    fn output_is_url_safe_base64_no_padding_43_chars() {
        let p = pseudonymize("PHONE", "03-1234-5678", "salt").unwrap();
        assert_eq!(p.len(), 43);
        assert!(!p.contains('='));
        assert!(!p.contains('+'));
        assert!(!p.contains('/'));
        // URL-safe 用の '-' / '_' は許容。base64 残りは英数字。
        assert!(p.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
    }

    /// 空 salt は拒否（仮名空間分離が機能しない）。
    #[test]
    fn empty_salt_rejected() {
        let err = pseudonymize("EMAIL", "x", "").unwrap_err();
        assert_eq!(err, PseudonymizeError::EmptySalt);
    }

    /// 空 value は拒否。
    #[test]
    fn empty_value_rejected() {
        let err = pseudonymize("EMAIL", "", "salt").unwrap_err();
        assert_eq!(err, PseudonymizeError::EmptyValue);
    }

    /// 空 field_type は拒否。
    #[test]
    fn empty_field_type_rejected() {
        let err = pseudonymize("", "x", "salt").unwrap_err();
        assert_eq!(err, PseudonymizeError::EmptyFieldType);
    }

    /// 不可逆性の弱い検証: 出力からは元値が復元できないこと（ハッシュ関数の標準性質、
    /// ここでは binding として「同一出力を生成する別 input 探索の困難性」を sanity check）。
    /// MAC 出力は 256-bit（32 byte）で、衝突は 2^128 試行相当。代表入力の出力は他の
    /// 入力と一致しない、という最低限の確認のみを行う。
    #[test]
    fn reasonable_collision_resistance() {
        let inputs = [
            ("EMAIL", "a@x.com"),
            ("EMAIL", "b@x.com"),
            ("EMAIL", "a@y.com"),
            ("PHONE", "03-1111-1111"),
            ("PHONE", "03-2222-2222"),
        ];
        let outs: Vec<String> = inputs
            .iter()
            .map(|(t, v)| pseudonymize(t, v, "salt").unwrap())
            .collect();
        for i in 0..outs.len() {
            for j in (i + 1)..outs.len() {
                assert_ne!(outs[i], outs[j], "collision detected at {} vs {}", i, j);
            }
        }
    }
}
