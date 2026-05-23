// jwt_helpers.rs — テスト用 compact JWT builder
// 署名を持たない compact JWT を構築するヘルパー。
// 検証ロジック（claims・format・timing）のテストに使用し、署名検証は対象外。

// base64: URL-safe パディングなし base64 エンコードに使用する
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
// base64 エンジントレイト
use base64::Engine;
// serde_json: ヘッダー/ペイロードの JSON シリアライズに使用する
use serde_json::{json, Value};

/// make_dpop_jwt は テスト用 DPoP proof compact JWT を構築する。
/// header_obj と payload_obj を JSON として base64url エンコードし、
/// フェイク署名を付与した compact JWT を返す。
pub fn make_dpop_jwt(header_obj: Value, payload_obj: Value) -> String {
    // ヘッダー JSON を base64url エンコードする
    let header_enc = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header_obj).unwrap());
    // ペイロード JSON を base64url エンコードする
    let payload_enc = URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload_obj).unwrap());
    // compact JWT 形式で連結する（署名部はテスト用のフェイク文字列）
    format!("{header_enc}.{payload_enc}.fakesignature")
}

/// dpop_header は標準的な DPoP proof JWT ヘッダーを返す。
pub fn dpop_header() -> Value {
    // RFC 9449 §4.2 の標準ヘッダー: typ=dpop+jwt, alg=ES256
    json!({
        "typ": "dpop+jwt",
        "alg": "ES256"
    })
}

/// dpop_payload は有効な DPoP proof payload を返す。
/// method: HTTP メソッド（大文字）、htu: target URI、jti: JWT ID
/// iat を現在 Unix time に設定する（期限内の proof）
pub fn dpop_payload(method: &str, htu: &str, jti: &str) -> Value {
    // 現在時刻を Unix timestamp として取得する（DPoP iat claim に使用する）
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // 有効な DPoP payload を構築する
    json!({
        "jti": jti,
        "htu": htu,
        "htm": method,
        "iat": now
    })
}

/// expired_dpop_payload は期限切れの DPoP proof payload を返す（iat が 120 秒前）。
pub fn expired_dpop_payload(method: &str, htu: &str, jti: &str) -> Value {
    // 現在時刻の 120 秒前を iat に設定する（60 秒 TTL を超えている）
    let stale = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .saturating_sub(120);
    // 期限切れ DPoP payload を構築する
    json!({
        "jti": jti,
        "htu": htu,
        "htm": method,
        "iat": stale
    })
}

/// make_oidc_claims_json は テスト用 OIDC claims payload を返す。
pub fn make_oidc_claims_json(
    sub: &str,
    iss: &str,
    aud: &str,
    exp_offset_secs: i64,
) -> Value {
    // 有効期限を現在時刻からのオフセットで計算する
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let exp = (now + exp_offset_secs).max(0) as u64;
    // OIDC claims JSON を構築する
    json!({
        "sub": sub,
        "jti": uuid::Uuid::new_v4().to_string(),
        "exp": exp,
        "iss": iss,
        "aud": aud,
        "scope": "openid profile"
    })
}

// uuid: OIDC claims の jti 生成に使用する
use uuid;
