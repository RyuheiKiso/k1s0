//! k1s0-auth 総合テスト
//!
//! JWT 検証、トークンリフレッシュ、OIDC フロー、権限検査のテスト。

use std::time::Duration;

/// RefreshToken のテスト
mod refresh_tests {
    use std::time::Duration;
    use k1s0_auth::refresh::{
        InMemoryRefreshTokenStore, RefreshTokenConfig, RefreshTokenData, RefreshTokenManager,
        RefreshTokenStore,
    };

    #[tokio::test]
    async fn test_issue_multiple_tokens_for_user() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default()
            .with_max_tokens_per_user(3);
        let manager = RefreshTokenManager::new(store, config);

        // 3 つのトークンを発行
        let token1 = manager.issue("user-1", Some("device-1".to_string())).await.unwrap();
        let token2 = manager.issue("user-1", Some("device-2".to_string())).await.unwrap();
        let token3 = manager.issue("user-1", Some("device-3".to_string())).await.unwrap();

        // すべてのトークンが有効
        assert!(manager.verify(&token1.token).await.is_ok());
        assert!(manager.verify(&token2.token).await.is_ok());
        assert!(manager.verify(&token3.token).await.is_ok());
    }

    #[tokio::test]
    async fn test_token_family_tracking() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        // 最初のトークンを発行
        let token1 = manager.issue("user-1", None).await.unwrap();
        let family_id = token1.data.family_id.clone();

        // ローテーション
        let (token2, _) = manager.rotate(&token1.token).await.unwrap();

        // 同じファミリー
        assert_eq!(token2.data.family_id, family_id);

        // 再度ローテーション
        let (token3, _) = manager.rotate(&token2.token).await.unwrap();
        assert_eq!(token3.data.family_id, family_id);
    }

    #[tokio::test]
    async fn test_revoke_family() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        let token = manager.issue("user-1", None).await.unwrap();
        let family_id = token.data.family_id.clone();

        // ローテーションで別のトークンも作成
        let (token2, _) = manager.rotate(&token.token).await.unwrap();

        // ファミリーを無効化
        let count = manager.revoke_family(&family_id).await.unwrap();
        assert!(count >= 1);

        // 両方のトークンが無効
        assert!(manager.verify(&token2.token).await.is_err());
    }

    #[tokio::test]
    async fn test_expired_token() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default()
            .with_token_ttl(Duration::from_millis(1)); // 非常に短い TTL
        let manager = RefreshTokenManager::new(store, config);

        let token = manager.issue("user-1", None).await.unwrap();

        // 少し待つ
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 期限切れ
        let result = manager.verify(&token.token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rotation_grace_period() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default()
            .with_rotation_grace_period(Duration::from_secs(60));
        let manager = RefreshTokenManager::new(store, config);

        let token = manager.issue("user-1", None).await.unwrap();

        // ローテーション
        let (_, _) = manager.rotate(&token.token).await.unwrap();

        // 猶予期間内なので古いトークンもまだ使える
        let result = manager.verify(&token.token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_device_info_preserved() {
        let store = InMemoryRefreshTokenStore::new();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(store, config);

        let device_info = "iPhone 15 Pro".to_string();
        let token = manager.issue("user-1", Some(device_info.clone())).await.unwrap();

        assert_eq!(token.data.device_info, Some(device_info.clone()));

        // ローテーション後も保持
        let (new_token, _) = manager.rotate(&token.token).await.unwrap();
        assert_eq!(new_token.data.device_info, Some(device_info));
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let store = InMemoryRefreshTokenStore::new();

        // 期限切れのトークンデータを直接追加
        let expired_data = RefreshTokenData {
            id: "expired-1".to_string(),
            user_id: "user-1".to_string(),
            family_id: "family-1".to_string(),
            issued_at: chrono::Utc::now().timestamp() - 86400,
            expires_at: chrono::Utc::now().timestamp() - 3600, // 1 時間前に期限切れ
            used: false,
            device_info: None,
            ip_address: None,
        };
        store.save("expired-token", &expired_data).await.unwrap();

        // 有効なトークン
        let valid_data = RefreshTokenData {
            id: "valid-1".to_string(),
            user_id: "user-1".to_string(),
            family_id: "family-2".to_string(),
            issued_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + 3600, // 1 時間後
            used: false,
            device_info: None,
            ip_address: None,
        };
        store.save("valid-token", &valid_data).await.unwrap();

        // クリーンアップ
        let cleaned = store.cleanup().await.unwrap();
        assert_eq!(cleaned, 1);

        // 期限切れは削除、有効は残る
        assert!(store.get("expired-token").await.unwrap().is_none());
        assert!(store.get("valid-token").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_tokens_by_user() {
        let store = InMemoryRefreshTokenStore::new();

        let data1 = RefreshTokenData {
            id: "token-1".to_string(),
            user_id: "user-1".to_string(),
            family_id: "family-1".to_string(),
            issued_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            used: false,
            device_info: None,
            ip_address: None,
        };
        store.save("token-1", &data1).await.unwrap();

        let data2 = RefreshTokenData {
            id: "token-2".to_string(),
            user_id: "user-1".to_string(),
            family_id: "family-2".to_string(),
            issued_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            used: false,
            device_info: None,
            ip_address: None,
        };
        store.save("token-2", &data2).await.unwrap();

        let data3 = RefreshTokenData {
            id: "token-3".to_string(),
            user_id: "user-2".to_string(),
            family_id: "family-3".to_string(),
            issued_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            used: false,
            device_info: None,
            ip_address: None,
        };
        store.save("token-3", &data3).await.unwrap();

        let user1_tokens = store.get_by_user("user-1").await.unwrap();
        assert_eq!(user1_tokens.len(), 2);

        let user2_tokens = store.get_by_user("user-2").await.unwrap();
        assert_eq!(user2_tokens.len(), 1);
    }
}

/// Policy 評価のテスト
mod policy_tests {
    use k1s0_auth::policy::{
        Action, PolicyBuilder, PolicyEvaluator, PolicyRequest, PolicySubject, ResourceContext,
    };

    #[tokio::test]
    async fn test_admin_bypass() {
        let evaluator = PolicyEvaluator::new();

        // 管理者ルールを追加
        let rules = PolicyBuilder::new()
            .admin_rule("admin")
            .build();
        evaluator.add_rules(rules).await;

        // 管理者はすべての操作が許可される
        let subject = PolicySubject::new("admin-user").with_role("admin");
        let action = Action::new("user", "delete");
        let request = PolicyRequest {
            subject,
            action,
            resource: ResourceContext::default(),
        };

        let result = evaluator.evaluate(&request).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_role_based_permission() {
        let evaluator = PolicyEvaluator::new();

        let rules = PolicyBuilder::new()
            .read_rule("user_read", "user", vec!["user"], 10)
            .write_rule("user_write", "user", vec!["editor"], 10)
            .build();
        evaluator.add_rules(rules).await;

        // user ロールは読み取りのみ可能
        let user_subject = PolicySubject::new("normal-user").with_role("user");

        let read_request = PolicyRequest {
            subject: user_subject.clone(),
            action: Action::new("user", "read"),
            resource: ResourceContext::default(),
        };
        assert!(evaluator.evaluate(&read_request).await.is_allowed());

        let write_request = PolicyRequest {
            subject: user_subject.clone(),
            action: Action::new("user", "write"),
            resource: ResourceContext::default(),
        };
        assert!(!evaluator.evaluate(&write_request).await.is_allowed());

        // editor ロールは書き込み可能
        let editor_subject = PolicySubject::new("editor-user").with_role("editor");
        let write_request = PolicyRequest {
            subject: editor_subject,
            action: Action::new("user", "write"),
            resource: ResourceContext::default(),
        };
        assert!(evaluator.evaluate(&write_request).await.is_allowed());
    }

    #[tokio::test]
    async fn test_multiple_roles() {
        let evaluator = PolicyEvaluator::new();

        let rules = PolicyBuilder::new()
            .read_rule("doc_read", "document", vec!["reader", "writer"], 10)
            .write_rule("doc_write", "document", vec!["writer"], 10)
            .build();
        evaluator.add_rules(rules).await;

        // 複数ロールを持つユーザー
        let subject = PolicySubject::new("power-user")
            .with_role("reader")
            .with_role("writer");

        let read_request = PolicyRequest {
            subject: subject.clone(),
            action: Action::new("document", "read"),
            resource: ResourceContext::default(),
        };
        assert!(evaluator.evaluate(&read_request).await.is_allowed());

        let write_request = PolicyRequest {
            subject: subject.clone(),
            action: Action::new("document", "write"),
            resource: ResourceContext::default(),
        };
        assert!(evaluator.evaluate(&write_request).await.is_allowed());
    }

    #[tokio::test]
    async fn test_deny_rule_priority() {
        let evaluator = PolicyEvaluator::new();

        // 拒否ルールは許可ルールより優先される
        let rules = PolicyBuilder::new()
            .read_rule("allow_read", "secret", vec!["user"], 10)
            .deny_rule("deny_secret", "secret", vec!["user"], 100)
            .build();
        evaluator.add_rules(rules).await;

        let subject = PolicySubject::new("user-1").with_role("user");
        let request = PolicyRequest {
            subject,
            action: Action::new("secret", "read"),
            resource: ResourceContext::default(),
        };

        let result = evaluator.evaluate(&request).await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_resource_owner_access() {
        let evaluator = PolicyEvaluator::new();

        let rules = PolicyBuilder::new()
            .owner_rule("owner_access", "profile", vec!["update", "delete"], 50)
            .build();
        evaluator.add_rules(rules).await;

        // オーナーは自分のリソースを操作できる
        let subject = PolicySubject::new("user-123");
        let resource = ResourceContext::default().with_owner("user-123");

        let request = PolicyRequest {
            subject: subject.clone(),
            action: Action::new("profile", "update"),
            resource: resource.clone(),
        };
        assert!(evaluator.evaluate(&request).await.is_allowed());

        // 他人のリソースは操作できない
        let other_resource = ResourceContext::default().with_owner("user-456");
        let request = PolicyRequest {
            subject,
            action: Action::new("profile", "update"),
            resource: other_resource,
        };
        assert!(!evaluator.evaluate(&request).await.is_allowed());
    }
}

/// Audit ログのテスト
mod audit_tests {
    use k1s0_auth::audit::{AuditActor, AuditEvent, AuditEventType, AuditLogger, AuditResult};

    #[test]
    fn test_audit_event_creation() {
        let actor = AuditActor::new("user-123")
            .with_ip_address("192.168.1.100")
            .with_user_agent("Mozilla/5.0");

        let event = AuditEvent::new(AuditEventType::Authentication, actor)
            .with_result(AuditResult::Success)
            .with_detail("Login successful");

        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.actor.id, "user-123");
        assert_eq!(event.actor.ip_address.as_deref(), Some("192.168.1.100"));
        assert_eq!(event.result, AuditResult::Success);
    }

    #[test]
    fn test_audit_actor_anonymous() {
        let actor = AuditActor::anonymous()
            .with_ip_address("10.0.0.1");

        assert!(actor.id.contains("anonymous"));
        assert_eq!(actor.ip_address.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn test_audit_event_types() {
        // すべてのイベントタイプをテスト
        let types = vec![
            AuditEventType::Authentication,
            AuditEventType::Authorization,
            AuditEventType::TokenIssuance,
            AuditEventType::TokenRefresh,
            AuditEventType::TokenRevocation,
            AuditEventType::PasswordChange,
            AuditEventType::AccountLockout,
            AuditEventType::PermissionChange,
        ];

        for event_type in types {
            let actor = AuditActor::new("test-user");
            let event = AuditEvent::new(event_type.clone(), actor);
            assert_eq!(event.event_type, event_type);
        }
    }

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::with_default_sink("test-service");

        // ログを記録（シンクへの書き込みはテストしない）
        let actor = AuditActor::new("user-1");
        logger.log_authentication_success(actor);
    }
}

/// Blacklist のテスト
mod blacklist_tests {
    use k1s0_auth::blacklist::{InMemoryBlacklist, TokenBlacklist};
    use std::time::Duration;

    #[tokio::test]
    async fn test_add_to_blacklist() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .add("token-123", Duration::from_secs(3600))
            .await
            .unwrap();

        assert!(blacklist.is_blacklisted("token-123").await.unwrap());
        assert!(!blacklist.is_blacklisted("token-456").await.unwrap());
    }

    #[tokio::test]
    async fn test_blacklist_expiration() {
        let blacklist = InMemoryBlacklist::new();

        // 短い TTL でブラックリストに追加
        blacklist
            .add("short-lived", Duration::from_millis(50))
            .await
            .unwrap();

        assert!(blacklist.is_blacklisted("short-lived").await.unwrap());

        // 期限切れまで待つ
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 自動的に期限切れになるかどうかは実装依存
        // ここでは cleanup を呼ぶ
        blacklist.cleanup().await.unwrap();

        assert!(!blacklist.is_blacklisted("short-lived").await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_from_blacklist() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .add("token-abc", Duration::from_secs(3600))
            .await
            .unwrap();

        assert!(blacklist.is_blacklisted("token-abc").await.unwrap());

        blacklist.remove("token-abc").await.unwrap();

        assert!(!blacklist.is_blacklisted("token-abc").await.unwrap());
    }

    #[tokio::test]
    async fn test_blacklist_count() {
        let blacklist = InMemoryBlacklist::new();

        blacklist
            .add("token-1", Duration::from_secs(3600))
            .await
            .unwrap();
        blacklist
            .add("token-2", Duration::from_secs(3600))
            .await
            .unwrap();
        blacklist
            .add("token-3", Duration::from_secs(3600))
            .await
            .unwrap();

        let count = blacklist.count().await.unwrap();
        assert_eq!(count, 3);
    }
}

/// JWT 検証のテスト
mod jwt_tests {
    use k1s0_auth::jwt::{Claims, JwtVerifier, JwtVerifierConfig};
    use std::time::Duration;

    #[test]
    fn test_claims_standard_fields() {
        let claims = Claims {
            sub: "user-123".to_string(),
            iss: Some("https://auth.example.com".to_string()),
            aud: Some(vec!["my-api".to_string()]),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            nbf: None,
            jti: Some("unique-jwt-id".to_string()),
            email: Some("user@example.com".to_string()),
            name: Some("Test User".to_string()),
            roles: vec!["user".to_string()],
            permissions: vec![],
            custom: Default::default(),
        };

        assert_eq!(claims.sub, "user-123");
        assert!(claims.has_role("user"));
        assert!(!claims.has_role("admin"));
    }

    #[test]
    fn test_claims_role_checking() {
        let claims = Claims {
            sub: "user-1".to_string(),
            iss: None,
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            email: None,
            name: None,
            roles: vec!["admin".to_string(), "user".to_string()],
            permissions: vec!["read:users".to_string(), "write:users".to_string()],
            custom: Default::default(),
        };

        assert!(claims.has_role("admin"));
        assert!(claims.has_role("user"));
        assert!(claims.has_any_role(&["admin", "superuser"]));
        assert!(claims.has_all_roles(&["admin", "user"]));
        assert!(!claims.has_all_roles(&["admin", "superuser"]));
    }

    #[test]
    fn test_claims_permission_checking() {
        let claims = Claims {
            sub: "user-1".to_string(),
            iss: None,
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            email: None,
            name: None,
            roles: vec![],
            permissions: vec!["read:users".to_string(), "write:users".to_string()],
            custom: Default::default(),
        };

        assert!(claims.has_permission("read:users"));
        assert!(claims.has_permission("write:users"));
        assert!(!claims.has_permission("delete:users"));
    }

    #[test]
    fn test_jwt_verifier_config() {
        let config = JwtVerifierConfig::new("https://auth.example.com")
            .with_jwks_uri("https://auth.example.com/.well-known/jwks.json")
            .with_audience("my-api")
            .with_clock_skew(Duration::from_secs(60));

        assert_eq!(config.issuer, "https://auth.example.com");
        assert_eq!(config.jwks_uri.as_deref(), Some("https://auth.example.com/.well-known/jwks.json"));
        assert_eq!(config.audience.as_deref(), Some("my-api"));
        assert_eq!(config.clock_skew, Duration::from_secs(60));
    }
}

/// OIDC 関連のテスト
mod oidc_tests {
    use k1s0_auth::oidc::{OidcConfig, OidcProviderConfig, UserInfo, UserInfoAddress};

    #[test]
    fn test_oidc_config() {
        let config = OidcConfig::new("https://auth.example.com")
            .with_client_id("my-client-id")
            .with_redirect_uri("https://myapp.example.com/callback")
            .with_scopes(vec!["openid", "profile", "email"]);

        assert_eq!(config.issuer, "https://auth.example.com");
        assert_eq!(config.client_id, "my-client-id");
        assert!(config.scopes.contains(&"openid".to_string()));
        assert!(config.scopes.contains(&"profile".to_string()));
        assert!(config.scopes.contains(&"email".to_string()));
    }

    #[test]
    fn test_user_info() {
        let user_info = UserInfo {
            sub: "user-123".to_string(),
            name: Some("Test User".to_string()),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            picture: Some("https://example.com/photo.jpg".to_string()),
            locale: Some("ja-JP".to_string()),
            address: None,
        };

        assert_eq!(user_info.sub, "user-123");
        assert_eq!(user_info.name.as_deref(), Some("Test User"));
        assert_eq!(user_info.email_verified, Some(true));
    }

    #[test]
    fn test_user_info_with_address() {
        let address = UserInfoAddress {
            formatted: Some("1-2-3 Example St, Tokyo, Japan".to_string()),
            street_address: Some("1-2-3 Example St".to_string()),
            locality: Some("Tokyo".to_string()),
            region: Some("Tokyo".to_string()),
            postal_code: Some("100-0001".to_string()),
            country: Some("Japan".to_string()),
        };

        let user_info = UserInfo {
            sub: "user-123".to_string(),
            name: None,
            given_name: None,
            family_name: None,
            email: None,
            email_verified: None,
            picture: None,
            locale: None,
            address: Some(address),
        };

        let addr = user_info.address.as_ref().unwrap();
        assert_eq!(addr.country.as_deref(), Some("Japan"));
        assert_eq!(addr.postal_code.as_deref(), Some("100-0001"));
    }

    #[test]
    fn test_oidc_provider_config() {
        let provider = OidcProviderConfig {
            issuer: "https://auth.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            userinfo_endpoint: Some("https://auth.example.com/userinfo".to_string()),
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            end_session_endpoint: Some("https://auth.example.com/logout".to_string()),
            revocation_endpoint: Some("https://auth.example.com/revoke".to_string()),
            scopes_supported: vec!["openid".to_string(), "profile".to_string()],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
        };

        assert_eq!(provider.issuer, "https://auth.example.com");
        assert!(provider.userinfo_endpoint.is_some());
        assert!(provider.scopes_supported.contains(&"openid".to_string()));
    }
}

/// Middleware のテスト
mod middleware_tests {
    use k1s0_auth::middleware::{AuthContext, AuthSkipMatcher};

    #[test]
    fn test_auth_context() {
        let context = AuthContext::new("user-123")
            .with_email("user@example.com")
            .with_roles(vec!["user".to_string(), "admin".to_string()])
            .with_tenant_id("tenant-1");

        assert_eq!(context.user_id(), "user-123");
        assert_eq!(context.email().as_deref(), Some("user@example.com"));
        assert!(context.has_role("admin"));
        assert!(!context.has_role("superuser"));
        assert_eq!(context.tenant_id().as_deref(), Some("tenant-1"));
    }

    #[test]
    fn test_auth_skip_matcher_paths() {
        let matcher = AuthSkipMatcher::new()
            .skip_path("/health")
            .skip_path("/metrics")
            .skip_prefix("/public/");

        assert!(matcher.should_skip("/health"));
        assert!(matcher.should_skip("/metrics"));
        assert!(matcher.should_skip("/public/file.txt"));
        assert!(!matcher.should_skip("/api/users"));
    }

    #[test]
    fn test_auth_skip_matcher_patterns() {
        let matcher = AuthSkipMatcher::new()
            .skip_pattern(r"^/v\d+/public/.*$");

        assert!(matcher.should_skip("/v1/public/docs"));
        assert!(matcher.should_skip("/v2/public/files"));
        assert!(!matcher.should_skip("/v1/private/users"));
    }
}
