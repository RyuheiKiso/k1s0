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

    pub fn kafka() -> Self {
        Self {
            image: "confluentinc/cp-kafka:7.7.0".to_string(),
            name: "kafka".to_string(),
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
            "kafka" => "localhost:9092".to_string(),
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

#[derive(Debug, Clone)]
pub struct PostgresContainer {
    handle: Option<TestContainerHandle>,
    db: String,
    user: String,
    password: String,
}

impl Default for PostgresContainer {
    fn default() -> Self {
        Self {
            handle: None,
            db: "test".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }
}

impl PostgresContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_db(mut self, db: impl Into<String>) -> Self {
        self.db = db.into();
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = user.into();
        self
    }

    pub async fn start(mut self) -> Self {
        let builder = TestContainerBuilder::postgres()
            .with_env("POSTGRES_DB", self.db.clone())
            .with_env("POSTGRES_USER", self.user.clone())
            .with_env("POSTGRES_PASSWORD", self.password.clone());
        self.handle = Some(builder.start().await);
        self
    }

    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@localhost:5432/{}",
            self.user, self.password, self.db
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct RedisContainer {
    handle: Option<TestContainerHandle>,
}

impl RedisContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(mut self) -> Self {
        self.handle = Some(TestContainerBuilder::redis().start().await);
        self
    }

    pub fn connection_url(&self) -> String {
        self.handle
            .as_ref()
            .map(|h| h.connection_url.clone())
            .unwrap_or_else(|| "redis://localhost:6379".to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct KafkaContainer {
    handle: Option<TestContainerHandle>,
}

impl KafkaContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(mut self) -> Self {
        self.handle = Some(TestContainerBuilder::kafka().start().await);
        self
    }

    pub fn bootstrap_servers(&self) -> String {
        self.handle
            .as_ref()
            .map(|h| h.connection_url.clone())
            .unwrap_or_else(|| "localhost:9092".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct KeycloakContainer {
    handle: Option<TestContainerHandle>,
    realm: String,
    admin_user: String,
    admin_password: String,
}

impl Default for KeycloakContainer {
    fn default() -> Self {
        Self {
            handle: None,
            realm: "master".to_string(),
            admin_user: "admin".to_string(),
            admin_password: "admin".to_string(),
        }
    }
}

impl KeycloakContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_realm(mut self, realm: impl Into<String>) -> Self {
        self.realm = realm.into();
        self
    }

    pub fn with_admin(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.admin_user = username.into();
        self.admin_password = password.into();
        self
    }

    pub async fn start(mut self) -> Self {
        let builder = TestContainerBuilder::keycloak()
            .with_env("KEYCLOAK_ADMIN", self.admin_user.clone())
            .with_env("KEYCLOAK_ADMIN_PASSWORD", self.admin_password.clone());
        self.handle = Some(builder.start().await);
        self
    }

    pub fn auth_url(&self) -> String {
        let base = self
            .handle
            .as_ref()
            .map(|h| h.connection_url.clone())
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        format!("{}/realms/{}", base.trim_end_matches('/'), self.realm)
    }
}
