//! JWKS endpoint tests using wiremock.
//! These tests verify that JwksVerifier correctly fetches keys from a JWKS endpoint.

#[cfg(test)]
mod jwks_wiremock_tests {
    use std::sync::Arc;
    use std::time::Duration;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use k1s0_auth::JwksVerifier;

    fn sample_jwks_response() -> serde_json::Value {
        serde_json::json!({
            "keys": [
                {
                    "kid": "test-key-1",
                    "kty": "RSA",
                    "alg": "RS256",
                    "use": "sig",
                    "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw",
                    "e": "AQAB"
                }
            ]
        })
    }

    #[tokio::test]
    async fn test_jwks_fetch_from_mock_endpoint() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_jwks_response()))
            .expect(1..)
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        let verifier = JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(60),
        );

        // Verify that an invalid token returns an error
        // (keys are fetched successfully but the token itself is invalid)
        let result = verifier.verify_token("invalid-jwt-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwks_fetch_failure_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        let verifier = JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(60),
        );

        let result = verifier.verify_token("some-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwks_cache_invalidation() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_jwks_response()))
            .expect(2..)
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        let verifier = Arc::new(JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(60),
        ));

        // First call fetches keys
        let _ = verifier.verify_token("invalid-token").await;

        // Invalidate cache
        verifier.invalidate_cache().await;

        // Second call should fetch keys again
        let _ = verifier.verify_token("invalid-token").await;
    }

    #[tokio::test]
    async fn test_jwks_empty_keys_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "keys": [] })),
            )
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        let verifier = JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(60),
        );

        // Should fail because no keys match
        let result = verifier.verify_token("invalid-jwt-token").await;
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // TTL キャッシュ動作テスト
    // -----------------------------------------------------------------------

    /// TTL = Duration::ZERO の場合、毎回 JWKS エンドポイントへフェッチすること。
    #[tokio::test]
    async fn test_jwks_ttl_zero_always_refetches() {
        let mock_server = MockServer::start().await;

        // 少なくとも 2 回フェッチされることを期待
        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_jwks_response()))
            .expect(2..)
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        // TTL = 0 → キャッシュは常に期限切れ → 毎回フェッチ
        let verifier = JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::ZERO,
        );

        let _ = verifier.verify_token("invalid-token").await;
        let _ = verifier.verify_token("invalid-token").await;
        // mock_server は Drop 時に expect(2..) を検証する
    }

    /// TTL が十分長い場合、2 回目以降はキャッシュを再利用すること（フェッチは 1 回のみ）。
    #[tokio::test]
    async fn test_jwks_long_ttl_reuses_cached_keys() {
        let mock_server = MockServer::start().await;

        // 正確に 1 回だけフェッチされることを期待
        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_jwks_response()))
            .expect(1..=1)
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        // TTL = 1 時間 → 2 回目はキャッシュを使用
        let verifier = JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(3600),
        );

        let _ = verifier.verify_token("invalid-token").await;
        let _ = verifier.verify_token("invalid-token").await;
        // mock_server は Drop 時に expect(1..=1) を検証する
    }

    /// 並行リクエストがあってもパニックせず、すべてのリクエストがエラーを返すこと。
    #[tokio::test]
    async fn test_jwks_concurrent_requests_no_panic() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/realms/k1s0/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_jwks_response()))
            .expect(1..)
            .mount(&mock_server)
            .await;

        let jwks_url = format!(
            "{}/realms/k1s0/protocol/openid-connect/certs",
            mock_server.uri()
        );

        let verifier = Arc::new(JwksVerifier::new(
            &jwks_url,
            "http://localhost:8180/realms/k1s0",
            "k1s0-api",
            Duration::from_secs(3600),
        ));

        // 4 つの並行リクエストを発行
        let v1 = verifier.clone();
        let v2 = verifier.clone();
        let v3 = verifier.clone();
        let v4 = verifier.clone();

        let (r1, r2, r3, r4) = tokio::join!(
            async move { v1.verify_token("invalid-token-1").await },
            async move { v2.verify_token("invalid-token-2").await },
            async move { v3.verify_token("invalid-token-3").await },
            async move { v4.verify_token("invalid-token-4").await },
        );

        // 無効なトークンなのでエラーになるが、パニックはしない
        assert!(r1.is_err());
        assert!(r2.is_err());
        assert!(r3.is_err());
        assert!(r4.is_err());
    }
}
