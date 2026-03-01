use std::sync::Arc;

use crate::domain::service::navigation_filter::{filter_navigation, FilteredNavigation, UserContext};
use crate::infrastructure::navigation_loader::NavigationConfigLoader;

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error("failed to load navigation config: {0}")]
    ConfigLoad(String),

    #[error("token verification failed: {0}")]
    TokenVerification(String),
}

pub struct GetNavigationUseCase {
    loader: Arc<dyn NavigationConfigLoader>,
    verifier: Option<Arc<k1s0_auth::JwksVerifier>>,
}

impl GetNavigationUseCase {
    pub fn new(
        loader: Arc<dyn NavigationConfigLoader>,
        verifier: Option<Arc<k1s0_auth::JwksVerifier>>,
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
            match verifier.verify_token(bearer_token).await {
                Ok(claims) => UserContext {
                    authenticated: true,
                    roles: claims.realm_roles().to_vec(),
                },
                Err(_) => UserContext {
                    authenticated: false,
                    roles: vec![],
                },
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
        assert!(matches!(result.unwrap_err(), NavigationError::ConfigLoad(_)));
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
}
