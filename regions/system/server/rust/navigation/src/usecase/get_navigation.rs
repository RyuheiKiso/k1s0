use std::sync::Arc;

use crate::domain::service::navigation_filter::{
    filter_navigation, FilteredNavigation, UserContext,
};
use crate::infrastructure::navigation_loader::NavigationConfigLoader;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait NavigationTokenVerifier: Send + Sync {
    async fn verify_roles(&self, bearer_token: &str) -> anyhow::Result<Vec<String>>;
}

pub struct JwksNavigationTokenVerifier {
    inner: Arc<k1s0_auth::JwksVerifier>,
}

impl JwksNavigationTokenVerifier {
    pub fn new(inner: Arc<k1s0_auth::JwksVerifier>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl NavigationTokenVerifier for JwksNavigationTokenVerifier {
    async fn verify_roles(&self, bearer_token: &str) -> anyhow::Result<Vec<String>> {
        let claims = self.inner.verify_token(bearer_token).await?;
        Ok(claims.realm_roles().to_vec())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error("failed to load navigation config: {0}")]
    ConfigLoad(String),

    #[error("token verification failed: {0}")]
    TokenVerification(String),
}

pub struct GetNavigationUseCase {
    loader: Arc<dyn NavigationConfigLoader>,
    verifier: Option<Arc<dyn NavigationTokenVerifier>>,
}

impl GetNavigationUseCase {
    pub fn new(
        loader: Arc<dyn NavigationConfigLoader>,
        verifier: Option<Arc<dyn NavigationTokenVerifier>>,
    ) -> Self {
        Self { loader, verifier }
    }

    pub async fn execute(&self, bearer_token: &str) -> Result<FilteredNavigation, NavigationError> {
        let config = self
            .loader
            .load()
            .map_err(|e| NavigationError::ConfigLoad(e.to_string()))?;

        let user_ctx = if bearer_token.is_empty() {
            UserContext {
                authenticated: false,
                roles: vec![],
            }
        } else if let Some(ref verifier) = self.verifier {
            match verifier.verify_roles(bearer_token).await {
                Ok(roles) => UserContext {
                    authenticated: true,
                    roles,
                },
                Err(err) => return Err(NavigationError::TokenVerification(err.to_string())),
            }
        } else {
            UserContext {
                authenticated: false,
                roles: vec![],
            }
        };

        Ok(filter_navigation(&config, &user_ctx))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::navigation::{Guard, GuardType, NavigationConfig, Route};
    use crate::infrastructure::navigation_loader::MockNavigationConfigLoader;

    fn make_config() -> NavigationConfig {
        NavigationConfig {
            version: 1,
            guards: vec![Guard {
                id: "auth_required".to_string(),
                guard_type: GuardType::AuthRequired,
                redirect_to: "/login".to_string(),
                roles: vec![],
            }],
            routes: vec![
                Route {
                    id: "public".to_string(),
                    path: "/".to_string(),
                    component_id: None,
                    guards: vec![],
                    transition: None,
                    transition_duration_ms: 300,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                Route {
                    id: "protected".to_string(),
                    path: "/dashboard".to_string(),
                    component_id: Some("Dashboard".to_string()),
                    guards: vec!["auth_required".to_string()],
                    transition: None,
                    transition_duration_ms: 300,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
            ],
        }
    }

    #[tokio::test]
    async fn empty_token_returns_public_routes() {
        let mut mock = MockNavigationConfigLoader::new();
        mock.expect_load().returning(|| Ok(make_config()));

        let uc = GetNavigationUseCase::new(Arc::new(mock), None);
        let result = uc.execute("").await.unwrap();
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"public"));
        assert!(!ids.contains(&"protected"));
    }

    #[tokio::test]
    async fn config_load_error() {
        let mut mock = MockNavigationConfigLoader::new();
        mock.expect_load()
            .returning(|| Err(anyhow::anyhow!("file not found")));

        let uc = GetNavigationUseCase::new(Arc::new(mock), None);
        let result = uc.execute("").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NavigationError::ConfigLoad(_)
        ));
    }

    #[tokio::test]
    async fn token_without_verifier_returns_public() {
        let mut mock = MockNavigationConfigLoader::new();
        mock.expect_load().returning(|| Ok(make_config()));

        let uc = GetNavigationUseCase::new(Arc::new(mock), None);
        let result = uc.execute("some-token").await.unwrap();
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"public"));
        assert!(!ids.contains(&"protected"));
    }

    #[tokio::test]
    async fn invalid_token_returns_verification_error() {
        let mut mock_loader = MockNavigationConfigLoader::new();
        mock_loader.expect_load().returning(|| Ok(make_config()));

        let mut mock_verifier = MockNavigationTokenVerifier::new();
        mock_verifier
            .expect_verify_roles()
            .withf(|token| token == "bad-token")
            .returning(|_| Err(anyhow::anyhow!("signature mismatch")));

        let uc = GetNavigationUseCase::new(Arc::new(mock_loader), Some(Arc::new(mock_verifier)));
        let result = uc.execute("bad-token").await;

        assert!(matches!(
            result,
            Err(NavigationError::TokenVerification(msg)) if msg.contains("signature mismatch")
        ));
    }
}
