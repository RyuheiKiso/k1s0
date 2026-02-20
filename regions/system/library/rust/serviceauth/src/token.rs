//! サービス間認証トークンと SPIFFE ワークロードアイデンティティ。

use crate::error::ServiceAuthError;
use chrono::{DateTime, Utc};

/// サービス間通信用アクセストークン。
///
/// Client Credentials フローで取得したトークン情報を保持し、
/// 有効期限チェックとリフレッシュタイミングの判定を提供する。
#[derive(Debug, Clone)]
pub struct ServiceToken {
    /// Bearer アクセストークン文字列。
    pub access_token: String,

    /// トークン種別（通常は "Bearer"）。
    pub token_type: String,

    /// トークンの有効期限（秒）。
    pub expires_in: u64,

    /// トークンを取得した時刻（UTC）。
    pub acquired_at: DateTime<Utc>,
}

impl ServiceToken {
    /// 新しい ServiceToken を生成する。
    pub fn new(access_token: String, token_type: String, expires_in: u64) -> Self {
        Self {
            access_token,
            token_type,
            expires_in,
            acquired_at: Utc::now(),
        }
    }

    /// トークンが有効期限切れかどうかを返す。
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.acquired_at)
            .num_seconds();
        elapsed < 0 || elapsed as u64 >= self.expires_in
    }

    /// 指定秒数前にリフレッシュすべきかどうかを返す。
    ///
    /// `refresh_before_secs` 秒以内に有効期限が切れる場合は `true` を返す。
    pub fn should_refresh(&self, refresh_before_secs: u64) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.acquired_at)
            .num_seconds();
        if elapsed < 0 {
            return false;
        }
        let elapsed_u64 = elapsed as u64;
        // expires_in より refresh_before_secs だけ早いタイミングを閾値とする
        elapsed_u64 + refresh_before_secs >= self.expires_in
    }

    /// Authorization ヘッダー用の Bearer 文字列を返す。
    pub fn bearer_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}

/// SPIFFE ワークロードアイデンティティ。
///
/// `spiffe://{trust_domain}/ns/{namespace}/sa/{service_account}` 形式の URI を解析・生成する。
#[derive(Debug, Clone, PartialEq)]
pub struct SpiffeId {
    /// トラストドメイン（例: `k1s0.internal`）。
    pub trust_domain: String,

    /// Kubernetes ネームスペース。
    pub namespace: String,

    /// サービスアカウント名。
    pub service_account: String,
}

impl SpiffeId {
    /// SPIFFE URI 文字列を解析して SpiffeId を返す。
    ///
    /// 期待フォーマット: `spiffe://{trust_domain}/ns/{namespace}/sa/{service_account}`
    ///
    /// # エラー
    ///
    /// フォーマットが不正な場合は `ServiceAuthError::SpiffeValidationFailed` を返す。
    pub fn parse(spiffe_uri: &str) -> Result<Self, ServiceAuthError> {
        let path = spiffe_uri
            .strip_prefix("spiffe://")
            .ok_or_else(|| {
                ServiceAuthError::SpiffeValidationFailed(format!(
                    "SPIFFE URI は 'spiffe://' で始まる必要があります: {}",
                    spiffe_uri
                ))
            })?;

        // trust_domain と残りのパスを分離する
        let (trust_domain, rest) = path.split_once('/').ok_or_else(|| {
            ServiceAuthError::SpiffeValidationFailed(format!(
                "SPIFFE URI にパスが含まれていません: {}",
                spiffe_uri
            ))
        })?;

        if trust_domain.is_empty() {
            return Err(ServiceAuthError::SpiffeValidationFailed(format!(
                "SPIFFE URI のトラストドメインが空です: {}",
                spiffe_uri
            )));
        }

        // /ns/{namespace}/sa/{service_account} を解析する
        let segments: Vec<&str> = rest.split('/').collect();
        if segments.len() != 4
            || segments[0] != "ns"
            || segments[2] != "sa"
            || segments[1].is_empty()
            || segments[3].is_empty()
        {
            return Err(ServiceAuthError::SpiffeValidationFailed(format!(
                "SPIFFE URI のパスは /ns/{{namespace}}/sa/{{service_account}} 形式である必要があります: {}",
                spiffe_uri
            )));
        }

        Ok(Self {
            trust_domain: trust_domain.to_string(),
            namespace: segments[1].to_string(),
            service_account: segments[3].to_string(),
        })
    }

    /// SPIFFE URI 文字列に変換する。
    pub fn to_uri(&self) -> String {
        format!(
            "spiffe://{}/ns/{}/sa/{}",
            self.trust_domain, self.namespace, self.service_account
        )
    }

    /// 指定した Tier（ネームスペース）のサービスへのアクセスを許可するかどうかを返す。
    ///
    /// k1s0 の Region 階層（system → business → service）に基づき、
    /// 上位 Tier のサービスは下位 Tier へのアクセスが許可される。
    ///
    /// | 自身の Tier | アクセス可能な Tier    |
    /// |------------|----------------------|
    /// | system     | system, business, service |
    /// | business   | business, service    |
    /// | service    | service のみ         |
    pub fn allows_tier_access(&self, target_tier: &str) -> bool {
        match self.namespace.as_str() {
            "system" => matches!(target_tier, "system" | "business" | "service"),
            "business" => matches!(target_tier, "business" | "service"),
            "service" => target_tier == "service",
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ServiceToken テスト ---

    fn make_token_with_acquired_at(expires_in: u64, acquired_at: DateTime<Utc>) -> ServiceToken {
        ServiceToken {
            access_token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in,
            acquired_at,
        }
    }

    #[test]
    fn test_is_expired_not_yet_expired() {
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        assert!(!token.is_expired());
    }

    #[test]
    fn test_is_expired_just_expired() {
        // 取得時刻を 901 秒前に設定する
        let acquired_at = Utc::now() - chrono::Duration::seconds(901);
        let token = make_token_with_acquired_at(900, acquired_at);
        assert!(token.is_expired());
    }

    #[test]
    fn test_is_expired_exactly_at_boundary() {
        // expires_in と経過時間が等しい場合は期限切れとみなす
        let acquired_at = Utc::now() - chrono::Duration::seconds(900);
        let token = make_token_with_acquired_at(900, acquired_at);
        assert!(token.is_expired());
    }

    #[test]
    fn test_should_refresh_far_from_expiry() {
        // 有効期限まで十分余裕がある場合はリフレッシュ不要
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        assert!(!token.should_refresh(120));
    }

    #[test]
    fn test_should_refresh_within_refresh_window() {
        // 有効期限の 120 秒前以内ならリフレッシュが必要
        let acquired_at = Utc::now() - chrono::Duration::seconds(800);
        let token = make_token_with_acquired_at(900, acquired_at);
        // 経過 800 秒、有効期限 900 秒、リフレッシュ閾値 120 秒
        // 800 + 120 = 920 >= 900 なのでリフレッシュが必要
        assert!(token.should_refresh(120));
    }

    #[test]
    fn test_should_refresh_exactly_at_boundary() {
        // elapsed + refresh_before_secs == expires_in の境界
        let acquired_at = Utc::now() - chrono::Duration::seconds(780);
        let token = make_token_with_acquired_at(900, acquired_at);
        // 780 + 120 = 900 >= 900 → リフレッシュが必要
        assert!(token.should_refresh(120));
    }

    #[test]
    fn test_should_refresh_just_before_boundary() {
        // elapsed + refresh_before_secs < expires_in → リフレッシュ不要
        // elapsed=779, 779 + 120 = 899 < 900 → false
        // ただしシステム時刻の誤差を考慮して少し余裕を持たせる
        let acquired_at = Utc::now() - chrono::Duration::seconds(770);
        let token = make_token_with_acquired_at(900, acquired_at);
        // 770 + 120 = 890 < 900 → リフレッシュ不要
        assert!(!token.should_refresh(120));
    }

    #[test]
    fn test_bearer_header() {
        let token = ServiceToken::new("my-access-token".to_string(), "Bearer".to_string(), 900);
        assert_eq!(token.bearer_header(), "Bearer my-access-token");
    }

    // --- SpiffeId テスト ---

    #[test]
    fn test_parse_valid_spiffe_id() {
        let uri = "spiffe://k1s0.internal/ns/system/sa/auth-service";
        let spiffe = SpiffeId::parse(uri).unwrap();

        assert_eq!(spiffe.trust_domain, "k1s0.internal");
        assert_eq!(spiffe.namespace, "system");
        assert_eq!(spiffe.service_account, "auth-service");
    }

    #[test]
    fn test_parse_business_namespace() {
        let uri = "spiffe://k1s0.internal/ns/business/sa/order-service";
        let spiffe = SpiffeId::parse(uri).unwrap();

        assert_eq!(spiffe.trust_domain, "k1s0.internal");
        assert_eq!(spiffe.namespace, "business");
        assert_eq!(spiffe.service_account, "order-service");
    }

    #[test]
    fn test_parse_missing_spiffe_prefix() {
        let result = SpiffeId::parse("https://k1s0.internal/ns/system/sa/svc");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_empty_string() {
        let result = SpiffeId::parse("");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_missing_path() {
        let result = SpiffeId::parse("spiffe://k1s0.internal");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_wrong_path_format() {
        // /ns/{ns}/sa/{sa} 形式でない場合
        let result = SpiffeId::parse("spiffe://k1s0.internal/system/auth-service");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_empty_namespace() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns//sa/auth-service");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_empty_service_account() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_parse_empty_trust_domain() {
        let result = SpiffeId::parse("spiffe:///ns/system/sa/auth-service");
        assert!(matches!(result, Err(ServiceAuthError::SpiffeValidationFailed(_))));
    }

    #[test]
    fn test_to_uri_roundtrip() {
        let original = "spiffe://k1s0.internal/ns/system/sa/auth-service";
        let spiffe = SpiffeId::parse(original).unwrap();
        assert_eq!(spiffe.to_uri(), original);
    }

    #[test]
    fn test_to_uri_format() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "business".to_string(),
            service_account: "payment-service".to_string(),
        };
        assert_eq!(
            spiffe.to_uri(),
            "spiffe://k1s0.internal/ns/business/sa/payment-service"
        );
    }

    #[test]
    fn test_allows_tier_access_system_to_all() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "system".to_string(),
            service_account: "auth-service".to_string(),
        };
        assert!(spiffe.allows_tier_access("system"));
        assert!(spiffe.allows_tier_access("business"));
        assert!(spiffe.allows_tier_access("service"));
    }

    #[test]
    fn test_allows_tier_access_business_to_business_and_service() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "business".to_string(),
            service_account: "order-service".to_string(),
        };
        assert!(!spiffe.allows_tier_access("system"));
        assert!(spiffe.allows_tier_access("business"));
        assert!(spiffe.allows_tier_access("service"));
    }

    #[test]
    fn test_allows_tier_access_service_to_service_only() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "service".to_string(),
            service_account: "leaf-service".to_string(),
        };
        assert!(!spiffe.allows_tier_access("system"));
        assert!(!spiffe.allows_tier_access("business"));
        assert!(spiffe.allows_tier_access("service"));
    }

    #[test]
    fn test_allows_tier_access_unknown_namespace() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "unknown".to_string(),
            service_account: "some-service".to_string(),
        };
        assert!(!spiffe.allows_tier_access("system"));
        assert!(!spiffe.allows_tier_access("business"));
        assert!(!spiffe.allows_tier_access("service"));
    }

    #[test]
    fn test_spiffe_id_equality() {
        let a = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
        let b = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
        let c = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/other-service").unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_service_token_new_sets_acquired_at() {
        let before = Utc::now();
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        let after = Utc::now();

        assert!(token.acquired_at >= before);
        assert!(token.acquired_at <= after);
    }
}
