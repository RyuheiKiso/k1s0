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
}
