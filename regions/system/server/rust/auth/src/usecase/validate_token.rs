use std::sync::Arc;

use crate::domain::entity::claims::Claims;
use crate::infrastructure::TokenVerifier;

/// AuthError はトークン検証に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("invalid issuer")]
    InvalidIssuer,

    #[error("invalid audience")]
    InvalidAudience,

    #[error("token expired")]
    TokenExpired,

    #[error("verification failed: {0}")]
    VerificationFailed(String),
}

/// ValidateTokenUseCase は JWT トークン検証ユースケース。
pub struct ValidateTokenUseCase {
    verifier: Arc<dyn TokenVerifier>,
    expected_issuer: String,
    expected_audience: String,
}

impl ValidateTokenUseCase {
    pub fn new(
        verifier: Arc<dyn TokenVerifier>,
        expected_issuer: String,
        expected_audience: String,
    ) -> Self {
        Self {
            verifier,
            expected_issuer,
            expected_audience,
        }
    }

    /// トークンを検証し、Claims を返却する。
    pub async fn execute(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self
            .verifier
            .verify_token(token)
            .await
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        // issuer の検証
        if claims.iss != self.expected_issuer {
            return Err(AuthError::InvalidIssuer);
        }

        // audience の検証
        if claims.aud != self.expected_audience {
            return Err(AuthError::InvalidAudience);
        }

        // 有効期限の検証
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::MockTokenVerifier;
    use std::collections::HashMap;

    fn make_valid_claims() -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: "k1s0-api".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: "token-uuid-5678".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid profile email".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            realm_access: crate::domain::entity::claims::RealmAccess {
                roles: vec!["user".to_string()],
            },
            resource_access: HashMap::new(),
            tier_access: vec!["system".to_string()],
        }
    }

    fn make_usecase(verifier: MockTokenVerifier) -> ValidateTokenUseCase {
        ValidateTokenUseCase::new(
            Arc::new(verifier),
            "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            "k1s0-api".to_string(),
        )
    }

    #[tokio::test]
    async fn test_validate_token_success() {
        let mut mock = MockTokenVerifier::new();
        let expected_claims = make_valid_claims();
        let return_claims = expected_claims.clone();

        mock.expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let uc = make_usecase(mock);
        let result = uc.execute("valid-token").await;
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, "user-uuid-1234");
        assert_eq!(claims.preferred_username, "taro.yamada");
    }

    #[tokio::test]
    async fn test_validate_token_invalid_token() {
        let mut mock = MockTokenVerifier::new();
        mock.expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("invalid signature")));

        let uc = make_usecase(mock);
        let result = uc.execute("invalid-token").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::InvalidToken(msg) => assert!(msg.contains("invalid signature")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_token_wrong_issuer() {
        let mut mock = MockTokenVerifier::new();
        let mut claims = make_valid_claims();
        claims.iss = "https://wrong-issuer.example.com".to_string();

        mock.expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let uc = make_usecase(mock);
        let result = uc.execute("token-with-wrong-issuer").await;
        assert!(matches!(result.unwrap_err(), AuthError::InvalidIssuer));
    }

    #[tokio::test]
    async fn test_validate_token_wrong_audience() {
        let mut mock = MockTokenVerifier::new();
        let mut claims = make_valid_claims();
        claims.aud = "wrong-audience".to_string();

        mock.expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let uc = make_usecase(mock);
        let result = uc.execute("token-with-wrong-audience").await;
        assert!(matches!(result.unwrap_err(), AuthError::InvalidAudience));
    }

    #[tokio::test]
    async fn test_validate_token_expired() {
        let mut mock = MockTokenVerifier::new();
        let mut claims = make_valid_claims();
        claims.exp = chrono::Utc::now().timestamp() - 3600; // expired 1 hour ago

        mock.expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let uc = make_usecase(mock);
        let result = uc.execute("expired-token").await;
        assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
    }
}
