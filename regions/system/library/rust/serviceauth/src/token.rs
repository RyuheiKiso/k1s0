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
        let path = spiffe_uri.strip_prefix("spiffe://").ok_or_else(|| {
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

    // トークンがまだ有効期限内であることを確認する。
    #[test]
    fn test_is_expired_not_yet_expired() {
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        assert!(!token.is_expired());
    }

    // トークンの有効期限が切れた直後に期限切れと判定されることを確認する。
    #[test]
    fn test_is_expired_just_expired() {
        // 取得時刻を 901 秒前に設定する
        let acquired_at = Utc::now() - chrono::Duration::seconds(901);
        let token = make_token_with_acquired_at(900, acquired_at);
        assert!(token.is_expired());
    }

    // 経過時間が expires_in と等しい境界値で期限切れとみなされることを確認する。
    #[test]
    fn test_is_expired_exactly_at_boundary() {
        // expires_in と経過時間が等しい場合は期限切れとみなす
        let acquired_at = Utc::now() - chrono::Duration::seconds(900);
        let token = make_token_with_acquired_at(900, acquired_at);
        assert!(token.is_expired());
    }

    // 有効期限まで十分余裕がある場合にリフレッシュ不要と判定されることを確認する。
    #[test]
    fn test_should_refresh_far_from_expiry() {
        // 有効期限まで十分余裕がある場合はリフレッシュ不要
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        assert!(!token.should_refresh(120));
    }

    // リフレッシュ閾値内にある場合にリフレッシュ必要と判定されることを確認する。
    #[test]
    fn test_should_refresh_within_refresh_window() {
        // 有効期限の 120 秒前以内ならリフレッシュが必要
        let acquired_at = Utc::now() - chrono::Duration::seconds(800);
        let token = make_token_with_acquired_at(900, acquired_at);
        // 経過 800 秒、有効期限 900 秒、リフレッシュ閾値 120 秒
        // 800 + 120 = 920 >= 900 なのでリフレッシュが必要
        assert!(token.should_refresh(120));
    }

    // elapsed + refresh_before_secs が expires_in と等しい境界でリフレッシュが必要なことを確認する。
    #[test]
    fn test_should_refresh_exactly_at_boundary() {
        // elapsed + refresh_before_secs == expires_in の境界
        let acquired_at = Utc::now() - chrono::Duration::seconds(780);
        let token = make_token_with_acquired_at(900, acquired_at);
        // 780 + 120 = 900 >= 900 → リフレッシュが必要
        assert!(token.should_refresh(120));
    }

    // リフレッシュ閾値を 1 秒下回る場合にリフレッシュ不要と判定されることを確認する。
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

    // bearer_header が "Bearer <token>" 形式の文字列を返すことを確認する。
    #[test]
    fn test_bearer_header() {
        let token = ServiceToken::new("my-access-token".to_string(), "Bearer".to_string(), 900);
        assert_eq!(token.bearer_header(), "Bearer my-access-token");
    }

    // --- SpiffeId テスト ---

    // 正しい SPIFFE URI を解析してフィールドが正しく設定されることを確認する。
    #[test]
    fn test_parse_valid_spiffe_id() {
        let uri = "spiffe://k1s0.internal/ns/system/sa/auth-service";
        let spiffe = SpiffeId::parse(uri).unwrap();

        assert_eq!(spiffe.trust_domain, "k1s0.internal");
        assert_eq!(spiffe.namespace, "system");
        assert_eq!(spiffe.service_account, "auth-service");
    }

    // business ネームスペースを含む SPIFFE URI が正しく解析されることを確認する。
    #[test]
    fn test_parse_business_namespace() {
        let uri = "spiffe://k1s0.internal/ns/business/sa/order-service";
        let spiffe = SpiffeId::parse(uri).unwrap();

        assert_eq!(spiffe.trust_domain, "k1s0.internal");
        assert_eq!(spiffe.namespace, "business");
        assert_eq!(spiffe.service_account, "order-service");
    }

    // "spiffe://" プレフィックスがない URI の解析がエラーになることを確認する。
    #[test]
    fn test_parse_missing_spiffe_prefix() {
        let result = SpiffeId::parse("https://k1s0.internal/ns/system/sa/svc");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // 空文字列の SPIFFE URI 解析がエラーになることを確認する。
    #[test]
    fn test_parse_empty_string() {
        let result = SpiffeId::parse("");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // パスが含まれない SPIFFE URI の解析がエラーになることを確認する。
    #[test]
    fn test_parse_missing_path() {
        let result = SpiffeId::parse("spiffe://k1s0.internal");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // /ns/{ns}/sa/{sa} 形式でないパスの解析がエラーになることを確認する。
    #[test]
    fn test_parse_wrong_path_format() {
        // /ns/{ns}/sa/{sa} 形式でない場合
        let result = SpiffeId::parse("spiffe://k1s0.internal/system/auth-service");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // ネームスペースが空の SPIFFE URI の解析がエラーになることを確認する。
    #[test]
    fn test_parse_empty_namespace() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns//sa/auth-service");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // サービスアカウントが空の SPIFFE URI の解析がエラーになることを確認する。
    #[test]
    fn test_parse_empty_service_account() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // トラストドメインが空の SPIFFE URI の解析がエラーになることを確認する。
    #[test]
    fn test_parse_empty_trust_domain() {
        let result = SpiffeId::parse("spiffe:///ns/system/sa/auth-service");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    // SPIFFE URI を解析して to_uri() で元の文字列に戻せることを確認する。
    #[test]
    fn test_to_uri_roundtrip() {
        let original = "spiffe://k1s0.internal/ns/system/sa/auth-service";
        let spiffe = SpiffeId::parse(original).unwrap();
        assert_eq!(spiffe.to_uri(), original);
    }

    // to_uri() が期待する SPIFFE URI フォーマットを生成することを確認する。
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

    // system ネームスペースが system/business/service すべての Tier にアクセスできることを確認する。
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

    // business ネームスペースが business と service の Tier にのみアクセスできることを確認する。
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

    // service ネームスペースが service Tier にのみアクセスできることを確認する。
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

    // 未知のネームスペースがいずれの Tier にもアクセスできないことを確認する。
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

    // 同一 SPIFFE URI から生成した SpiffeId が等しく、異なる URI なら等しくないことを確認する。
    #[test]
    fn test_spiffe_id_equality() {
        let a = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
        let b = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
        let c = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/other-service").unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ServiceToken::new が acquired_at を現在時刻に設定することを確認する。
    #[test]
    fn test_service_token_new_sets_acquired_at() {
        let before = Utc::now();
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 900);
        let after = Utc::now();

        assert!(token.acquired_at >= before);
        assert!(token.acquired_at <= after);
    }

    // ServiceToken の Clone が全フィールドを正しくコピーすることを確認する。
    #[test]
    fn test_service_token_clone() {
        let token = ServiceToken::new("my-token-123".to_string(), "Bearer".to_string(), 3600);
        let cloned = token.clone();
        assert_eq!(cloned.access_token, token.access_token);
        assert_eq!(cloned.token_type, token.token_type);
        assert_eq!(cloned.expires_in, token.expires_in);
        assert_eq!(cloned.acquired_at, token.acquired_at);
    }

    // bearer_header が token_type に関わらず "Bearer" を使用することを確認する。
    #[test]
    fn test_bearer_header_format() {
        let token = ServiceToken::new("abc-def-ghi".to_string(), "bearer".to_string(), 900);
        // bearer_header は常に "Bearer {access_token}" 形式
        assert_eq!(token.bearer_header(), "Bearer abc-def-ghi");
    }

    // expires_in が 0 の場合にすぐに期限切れと判定されることを確認する。
    #[test]
    fn test_is_expired_zero_expires_in() {
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 0);
        assert!(token.is_expired());
    }

    // expires_in が 0 の場合に should_refresh が true を返すことを確認する。
    #[test]
    fn test_should_refresh_zero_expires_in() {
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 0);
        assert!(token.should_refresh(0));
    }

    // refresh_before_secs が 0 の場合でも期限切れ直前はリフレッシュ不要と判定されることを確認する。
    #[test]
    fn test_should_refresh_zero_refresh_before() {
        let token = ServiceToken::new("tok".to_string(), "Bearer".to_string(), 3600);
        // refresh_before_secs=0 なら期限切れぎりぎりまでリフレッシュ不要
        assert!(!token.should_refresh(0));
    }

    // SpiffeId の to_uri が正しい SPIFFE URI を生成することを確認する。
    #[test]
    fn test_spiffe_id_to_uri_service_namespace() {
        let spiffe = SpiffeId {
            trust_domain: "example.com".to_string(),
            namespace: "service".to_string(),
            service_account: "api-gateway".to_string(),
        };
        assert_eq!(
            spiffe.to_uri(),
            "spiffe://example.com/ns/service/sa/api-gateway"
        );
    }

    // SpiffeId の parse が余分なパスセグメントを含む URI を拒否することを確認する。
    #[test]
    fn test_spiffe_parse_extra_segments() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service/extra");
        assert!(result.is_err());
    }

    // SpiffeId の parse が "ns" でなく別のプレフィックスを含む URI を拒否することを確認する。
    #[test]
    fn test_spiffe_parse_wrong_ns_prefix() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/namespace/system/sa/auth-service");
        assert!(result.is_err());
    }

    // SpiffeId の parse が "sa" でなく別のプレフィックスを含む URI を拒否することを確認する。
    #[test]
    fn test_spiffe_parse_wrong_sa_prefix() {
        let result = SpiffeId::parse("spiffe://k1s0.internal/ns/system/service-account/auth-service");
        assert!(result.is_err());
    }

    // allows_tier_access が未知の target_tier に対して false を返すことを確認する。
    #[test]
    fn test_allows_tier_access_unknown_target() {
        let spiffe = SpiffeId {
            trust_domain: "k1s0.internal".to_string(),
            namespace: "system".to_string(),
            service_account: "auth-service".to_string(),
        };
        assert!(!spiffe.allows_tier_access("unknown-tier"));
    }

    // SpiffeId の Clone が正しく動作することを確認する。
    #[test]
    fn test_spiffe_id_clone() {
        let original = SpiffeId::parse("spiffe://k1s0.internal/ns/system/sa/auth-service").unwrap();
        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(cloned.trust_domain, "k1s0.internal");
        assert_eq!(cloned.namespace, "system");
        assert_eq!(cloned.service_account, "auth-service");
    }
}
