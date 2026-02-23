use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

/// テスト用 JWT クレーム。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

impl Default for TestClaims {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            sub: String::new(),
            roles: Vec::new(),
            tenant_id: None,
            iat: now.timestamp(),
            exp: (now + Duration::hours(1)).timestamp(),
        }
    }
}

/// テスト用 JWT トークン生成ヘルパー (HS256 簡易実装)。
///
/// テスト用途のため、Base64URL エンコードのみで署名は HMAC-SHA256 の代わりに
/// secret をそのまま付加する簡易方式を採用する。
pub struct JwtTestHelper {
    secret: String,
}

impl JwtTestHelper {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    /// 管理者トークンを生成する。
    pub fn create_admin_token(&self) -> String {
        let claims = TestClaims {
            sub: "admin".to_string(),
            roles: vec!["admin".to_string()],
            ..Default::default()
        };
        self.create_token(&claims)
    }

    /// ユーザートークンを生成する。
    pub fn create_user_token(&self, user_id: &str, roles: Vec<String>) -> String {
        let claims = TestClaims {
            sub: user_id.to_string(),
            roles,
            ..Default::default()
        };
        self.create_token(&claims)
    }

    /// カスタムクレームでトークンを生成する。
    ///
    /// テスト用簡易 JWT: header.payload.signature の形式。
    pub fn create_token(&self, claims: &TestClaims) -> String {
        let header = base64url_encode(r#"{"alg":"HS256","typ":"JWT"}"#.as_bytes());
        let payload_json = serde_json::to_string(claims).expect("claims serialization");
        let payload = base64url_encode(payload_json.as_bytes());
        let signing_input = format!("{}.{}", header, payload);
        let signature = base64url_encode(
            format!("{}:{}", signing_input, self.secret).as_bytes(),
        );
        format!("{}.{}", signing_input, signature)
    }

    /// トークンのペイロードをデコードしてクレームを返す。
    pub fn decode_claims(&self, token: &str) -> Option<TestClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let payload_bytes = base64url_decode(parts[1])?;
        serde_json::from_slice(&payload_bytes).ok()
    }
}

fn base64url_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::new();
    static TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut i = 0;
    while i + 2 < data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        let _ = write!(
            result,
            "{}{}{}{}",
            TABLE[(n >> 18 & 63) as usize] as char,
            TABLE[(n >> 12 & 63) as usize] as char,
            TABLE[(n >> 6 & 63) as usize] as char,
            TABLE[(n & 63) as usize] as char,
        );
        i += 3;
    }
    let remaining = data.len() - i;
    if remaining == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        let _ = write!(
            result,
            "{}{}{}",
            TABLE[(n >> 18 & 63) as usize] as char,
            TABLE[(n >> 12 & 63) as usize] as char,
            TABLE[(n >> 6 & 63) as usize] as char,
        );
    } else if remaining == 1 {
        let n = (data[i] as u32) << 16;
        let _ = write!(
            result,
            "{}{}",
            TABLE[(n >> 18 & 63) as usize] as char,
            TABLE[(n >> 12 & 63) as usize] as char,
        );
    }
    result
}

fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    let bytes = input.as_bytes();
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in bytes {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'-' => 62,
            b'_' => 63,
            b'=' => continue,
            _ => return None,
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            data.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_admin_token() {
        let helper = JwtTestHelper::new("test-secret");
        let token = helper.create_admin_token();
        assert!(token.contains('.'));
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_create_user_token() {
        let helper = JwtTestHelper::new("test-secret");
        let token = helper.create_user_token("user-123", vec!["user".to_string()]);
        let claims = helper.decode_claims(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.roles, vec!["user"]);
    }

    #[test]
    fn test_create_token_with_tenant() {
        let helper = JwtTestHelper::new("secret");
        let claims = TestClaims {
            sub: "svc".to_string(),
            roles: vec!["service".to_string()],
            tenant_id: Some("t-1".to_string()),
            ..Default::default()
        };
        let token = helper.create_token(&claims);
        let decoded = helper.decode_claims(&token).unwrap();
        assert_eq!(decoded.tenant_id, Some("t-1".to_string()));
    }

    #[test]
    fn test_decode_invalid_token() {
        let helper = JwtTestHelper::new("s");
        assert!(helper.decode_claims("invalid").is_none());
    }

    #[test]
    fn test_default_claims_expiry() {
        let claims = TestClaims::default();
        assert!(claims.exp > claims.iat);
        assert!(claims.exp - claims.iat >= 3599);
    }
}
