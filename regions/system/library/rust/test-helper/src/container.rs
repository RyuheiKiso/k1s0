/// Thin testcontainer builder facade used by integration tests.
///
/// This intentionally focuses on connection metadata and leaves actual container
/// lifecycle integration to feature-specific tests.
#[derive(Debug, Clone)]
pub struct TestContainerBuilder {
    image: String,
    name: String,
    env: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct TestContainerHandle {
    pub name: String,
    pub image: String,
    pub connection_url: String,
}

impl TestContainerBuilder {
    pub fn postgres() -> Self {
        Self {
            image: "postgres:16-alpine".to_string(),
            name: "postgres".to_string(),
            env: vec![
                ("POSTGRES_USER".to_string(), "postgres".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ("POSTGRES_DB".to_string(), "test".to_string()),
            ],
        }
    }

    pub fn redis() -> Self {
        Self {
            image: "redis:7-alpine".to_string(),
            name: "redis".to_string(),
            env: Vec::new(),
        }
    }

    pub fn keycloak() -> Self {
        Self {
            image: "quay.io/keycloak/keycloak:26.0".to_string(),
            name: "keycloak".to_string(),
            env: vec![
                ("KEYCLOAK_ADMIN".to_string(), "admin".to_string()),
                ("KEYCLOAK_ADMIN_PASSWORD".to_string(), "admin".to_string()),
            ],
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub async fn start(self) -> TestContainerHandle {
        let connection_url = match self.name.as_str() {
            "postgres" => "postgresql://postgres:postgres@localhost:5432/test".to_string(),
            "redis" => "redis://localhost:6379".to_string(),
            "keycloak" => "http://localhost:8080".to_string(),
            _ => "http://localhost".to_string(),
        };

        TestContainerHandle {
            name: self.name,
            image: self.image,
            connection_url,
        }
    }
}
