//! テスト: JWT JWKS 検証 + RBAC

#[cfg(test)]
mod tests {
    use crate::claims::{Access, Audience, Claims, RealmAccess};
    use crate::rbac::{has_permission, has_resource_role, has_role, has_tier_access};
    use crate::verifier::{AuthError, JwkKey, JwksFetcher, JwksVerifier};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use rand::rngs::OsRng;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::traits::PublicKeyParts;
    use rsa::RsaPrivateKey;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const TEST_ISSUER: &str = "https://auth.k1s0.internal.example.com/realms/k1s0";
    const TEST_AUDIENCE: &str = "k1s0-api";
    const TEST_KID: &str = "test-key-1";

    /// テスト用の RSA 鍵ペアを生成する。
    fn generate_test_keypair() -> (RsaPrivateKey, JwkKey) {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

        let jwk_key = JwkKey {
            kid: TEST_KID.into(),
            n,
            e,
        };

        (private_key, jwk_key)
    }

    /// テスト用の Claims 構造体（jsonwebtoken 用のシリアライズ可能な形式）。
    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        iss: String,
        aud: String,
        exp: u64,
        iat: u64,
        jti: String,
        typ: String,
        azp: String,
        scope: String,
        preferred_username: String,
        email: String,
        realm_access: TestRealmAccess,
        resource_access: HashMap<String, TestAccess>,
        tier_access: Vec<String>,
    }

    #[derive(Serialize)]
    struct TestRealmAccess {
        roles: Vec<String>,
    }

    #[derive(Serialize)]
    struct TestAccess {
        roles: Vec<String>,
    }

    /// テスト用の JWT トークンを生成する。
    fn generate_test_token(
        private_key: &RsaPrivateKey,
        claims_override: Option<TestClaims>,
    ) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = claims_override.unwrap_or_else(|| {
            let mut resource_access = HashMap::new();
            resource_access.insert(
                "order-service".into(),
                TestAccess {
                    roles: vec!["read".into(), "write".into()],
                },
            );

            TestClaims {
                sub: "user-uuid-1234".into(),
                iss: TEST_ISSUER.into(),
                aud: TEST_AUDIENCE.into(),
                exp: now + 900, // 15分後
                iat: now,
                jti: "token-uuid-5678".into(),
                typ: "Bearer".into(),
                azp: "react-spa".into(),
                scope: "openid profile email".into(),
                preferred_username: "taro.yamada".into(),
                email: "taro.yamada@example.com".into(),
                realm_access: TestRealmAccess {
                    roles: vec!["user".into(), "order_manager".into()],
                },
                resource_access,
                tier_access: vec!["system".into(), "business".into(), "service".into()],
            }
        });

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.into());

        let pem = private_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .unwrap();
        let key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();

        encode(&header, &claims, &key).unwrap()
    }

    /// モック JWKS フェッチャー。
    struct MockFetcher {
        keys: Vec<JwkKey>,
    }

    #[async_trait::async_trait]
    impl JwksFetcher for MockFetcher {
        async fn fetch_keys(&self, _jwks_url: &str) -> Result<Vec<JwkKey>, AuthError> {
            Ok(self.keys.clone())
        }
    }

    /// フェッチ回数を記録するフェッチャー。
    struct CountingFetcher {
        inner: MockFetcher,
        count: Arc<tokio::sync::Mutex<u32>>,
    }

    #[async_trait::async_trait]
    impl JwksFetcher for CountingFetcher {
        async fn fetch_keys(&self, jwks_url: &str) -> Result<Vec<JwkKey>, AuthError> {
            let mut count = self.count.lock().await;
            *count += 1;
            self.inner.fetch_keys(jwks_url).await
        }
    }

    // --- Claims テスト ---

    #[test]
    fn test_claims_display() {
        let claims = Claims {
            sub: "user-1".into(),
            iss: TEST_ISSUER.into(),
            aud: Audience(vec![TEST_AUDIENCE.into()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro".into()),
            email: Some("taro@example.com".into()),
            realm_access: None,
            resource_access: None,
            tier_access: None,
        };

        let s = format!("{}", claims);
        assert!(s.contains("user-1"));
        assert!(s.contains("taro"));
    }

    #[test]
    fn test_claims_audience() {
        let claims = Claims {
            sub: "user-1".into(),
            iss: "iss".into(),
            aud: Audience(vec!["aud1".into(), "aud2".into()]),
            exp: 0,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: None,
            resource_access: None,
            tier_access: None,
        };

        assert_eq!(claims.audience(), Some("aud1"));
    }

    #[test]
    fn test_claims_realm_roles() {
        let claims = Claims {
            sub: "user-1".into(),
            iss: "iss".into(),
            aud: Audience(vec![]),
            exp: 0,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: Some(RealmAccess {
                roles: vec!["user".into(), "admin".into()],
            }),
            resource_access: None,
            tier_access: None,
        };

        assert_eq!(claims.realm_roles(), &["user", "admin"]);
    }

    #[test]
    fn test_claims_resource_roles() {
        let mut ra = HashMap::new();
        ra.insert(
            "order-service".to_string(),
            Access {
                roles: vec!["read".into(), "write".into()],
            },
        );

        let claims = Claims {
            sub: "user-1".into(),
            iss: "iss".into(),
            aud: Audience(vec![]),
            exp: 0,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: None,
            resource_access: Some(ra),
            tier_access: None,
        };

        assert_eq!(claims.resource_roles("order-service"), &["read", "write"]);
        assert!(claims.resource_roles("user-service").is_empty());
    }

    // --- JwksVerifier テスト ---

    #[tokio::test]
    async fn test_verify_token_success() {
        let (priv_key, jwk_key) = generate_test_keypair();
        let token = generate_test_token(&priv_key, None);

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let claims = verifier.verify_token(&token).await.unwrap();
        assert_eq!(claims.sub, "user-uuid-1234");
        assert_eq!(claims.iss, TEST_ISSUER);
        assert_eq!(claims.audience(), Some(TEST_AUDIENCE));
        assert_eq!(claims.preferred_username.as_deref(), Some("taro.yamada"));
        assert_eq!(claims.email.as_deref(), Some("taro.yamada@example.com"));
    }

    #[tokio::test]
    async fn test_verify_token_expired() {
        let (priv_key, jwk_key) = generate_test_keypair();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = TestClaims {
            sub: "user-1".into(),
            iss: TEST_ISSUER.into(),
            aud: TEST_AUDIENCE.into(),
            exp: now - 3600, // 1時間前に期限切れ
            iat: now - 7200,
            jti: "jti".into(),
            typ: "Bearer".into(),
            azp: "test".into(),
            scope: "openid".into(),
            preferred_username: "user".into(),
            email: "user@example.com".into(),
            realm_access: TestRealmAccess { roles: vec![] },
            resource_access: HashMap::new(),
            tier_access: vec![],
        };

        let token = generate_test_token(&priv_key, Some(claims));

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let result = verifier.verify_token(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token_wrong_issuer() {
        let (priv_key, jwk_key) = generate_test_keypair();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = TestClaims {
            sub: "user-1".into(),
            iss: "https://evil.example.com/realms/bad".into(),
            aud: TEST_AUDIENCE.into(),
            exp: now + 900,
            iat: now,
            jti: "jti".into(),
            typ: "Bearer".into(),
            azp: "test".into(),
            scope: "openid".into(),
            preferred_username: "user".into(),
            email: "user@example.com".into(),
            realm_access: TestRealmAccess { roles: vec![] },
            resource_access: HashMap::new(),
            tier_access: vec![],
        };

        let token = generate_test_token(&priv_key, Some(claims));

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let result = verifier.verify_token(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token_wrong_audience() {
        let (priv_key, jwk_key) = generate_test_keypair();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = TestClaims {
            sub: "user-1".into(),
            iss: TEST_ISSUER.into(),
            aud: "wrong-audience".into(),
            exp: now + 900,
            iat: now,
            jti: "jti".into(),
            typ: "Bearer".into(),
            azp: "test".into(),
            scope: "openid".into(),
            preferred_username: "user".into(),
            email: "user@example.com".into(),
            realm_access: TestRealmAccess { roles: vec![] },
            resource_access: HashMap::new(),
            tier_access: vec![],
        };

        let token = generate_test_token(&priv_key, Some(claims));

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let result = verifier.verify_token(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token_invalid_token() {
        let (_, jwk_key) = generate_test_keypair();

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let result = verifier.verify_token("invalid-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let (priv_key, jwk_key) = generate_test_keypair();
        let token = generate_test_token(&priv_key, None);

        let count = Arc::new(tokio::sync::Mutex::new(0u32));
        let fetcher = CountingFetcher {
            inner: MockFetcher {
                keys: vec![jwk_key],
            },
            count: count.clone(),
        };

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(fetcher),
        );

        // 1回目: フェッチが発生
        verifier.verify_token(&token).await.unwrap();
        assert_eq!(*count.lock().await, 1);

        // 2回目: キャッシュから取得
        verifier.verify_token(&token).await.unwrap();
        assert_eq!(*count.lock().await, 1); // フェッチ回数は増えない
    }

    #[tokio::test]
    async fn test_invalidate_cache() {
        let (priv_key, jwk_key) = generate_test_keypair();
        let token = generate_test_token(&priv_key, None);

        let count = Arc::new(tokio::sync::Mutex::new(0u32));
        let fetcher = CountingFetcher {
            inner: MockFetcher {
                keys: vec![jwk_key],
            },
            count: count.clone(),
        };

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(fetcher),
        );

        verifier.verify_token(&token).await.unwrap();
        assert_eq!(*count.lock().await, 1);

        // キャッシュを無効化
        verifier.invalidate_cache().await;

        verifier.verify_token(&token).await.unwrap();
        assert_eq!(*count.lock().await, 2); // 再フェッチが発生
    }

    // --- RBAC テスト (verifier 経由) ---

    #[tokio::test]
    async fn test_verify_and_check_permission() {
        let (priv_key, jwk_key) = generate_test_keypair();
        let token = generate_test_token(&priv_key, None);

        let verifier = JwksVerifier::with_fetcher(
            "https://auth.example.com/jwks",
            TEST_ISSUER,
            TEST_AUDIENCE,
            Duration::from_secs(600),
            Arc::new(MockFetcher {
                keys: vec![jwk_key],
            }),
        );

        let claims = verifier.verify_token(&token).await.unwrap();

        // RBAC チェック
        assert!(has_role(&claims, "user"));
        assert!(has_role(&claims, "order_manager"));
        assert!(!has_role(&claims, "sys_admin"));

        assert!(has_resource_role(&claims, "order-service", "read"));
        assert!(has_resource_role(&claims, "order-service", "write"));
        assert!(!has_resource_role(&claims, "order-service", "delete"));

        assert!(has_permission(&claims, "order-service", "read"));
        assert!(!has_permission(&claims, "order-service", "delete"));

        assert!(has_tier_access(&claims, "system"));
        assert!(has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "service"));
    }
}
