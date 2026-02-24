# system-server 実装設計

system-server（認証サーバー）の Rust 実装詳細を定義する。概要・API 定義・アーキテクチャは [system-server設計.md](system-server設計.md) を参照。

---

## Rust 実装 (regions/system/server/rust/auth/)

### ディレクトリ構成

```
regions/system/server/rust/auth/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs                  # User エンティティ
│   │   │   ├── role.rs                  # Role エンティティ
│   │   │   ├── permission.rs            # Permission エンティティ
│   │   │   └── audit_log.rs             # AuditLog エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── user_repository.rs       # UserRepository トレイト
│   │   │   └── audit_log_repository.rs  # AuditLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── auth_domain_service.rs   # パーミッション解決ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── validate_token.rs
│   │   ├── get_user.rs
│   │   ├── list_users.rs
│   │   ├── get_user_roles.rs
│   │   ├── check_permission.rs
│   │   ├── record_audit_log.rs
│   │   └── search_audit_log.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── rest_handler.rs          # axum REST ハンドラー
│   │   │   ├── grpc_handler.rs          # tonic gRPC ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   ├── gateway/
│   │   │   ├── mod.rs
│   │   │   └── keycloak_client.rs       # Keycloak Admin API クライアント
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── logger.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   └── audit_log_repository.rs
│       ├── auth/
│       │   ├── mod.rs
│       │   └── jwks.rs                  # JWKS 検証
│       └── messaging/
│           ├── mod.rs
│           └── producer.rs              # Kafka プロデューサー
├── api/
│   └── proto/
│       └── k1s0/system/auth/v1/
│           └── auth.proto
├── migrations/
│   └── 001_create_audit_logs.sql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs                             # tonic-build（proto コンパイル）
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

### Cargo.toml

```toml
[package]
name = "auth-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = { version = "1", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# JWT
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# OpenTelemetry
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"

# ユーティリティ
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
validator = { version = "0.18", features = ["derive"] }

# メトリクス
prometheus = "0.13"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
testcontainers = "0.23"

[build-dependencies]
tonic-build = "0.12"
```

### build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["api/proto/k1s0/system/auth/v1/auth.proto"],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

### src/main.rs

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tonic::transport::Server as TonicServer;
use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::gateway::KeycloakClient;
use adapter::handler::{grpc_handler, rest_handler};
use domain::service::AuthDomainService;
use infrastructure::auth::JwksVerifier;
use infrastructure::config::Config;
use infrastructure::messaging::KafkaProducer;
use infrastructure::persistence;
use usecase::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Config ---
    let cfg = Config::load("config/config.yaml")?;
    cfg.validate()?;

    // --- Logger ---
    infrastructure::config::init_logger(&cfg.app.environment);

    // --- OpenTelemetry ---
    let _tracer = infrastructure::config::init_tracer(&cfg.app.name)?;

    // --- Database ---
    let pool = persistence::connect(&cfg.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // --- Kafka ---
    let producer = KafkaProducer::new(&cfg.kafka)?;

    // --- DI ---
    let jwks_verifier = Arc::new(
        JwksVerifier::new(
            cfg.auth.oidc.jwks_uri.clone(),
            cfg.auth.oidc.jwks_cache_ttl(),
        ),
    );
    let keycloak_client = Arc::new(KeycloakClient::new(&cfg.auth.oidc));
    let audit_repo = Arc::new(persistence::AuditLogRepositoryImpl::new(pool.clone()));
    let auth_domain_svc = Arc::new(AuthDomainService::new());

    let validate_token_uc = ValidateTokenUseCase::new(
        jwks_verifier.clone(), cfg.auth.jwt.clone(),
    );
    let get_user_uc = GetUserUseCase::new(keycloak_client.clone());
    let list_users_uc = ListUsersUseCase::new(keycloak_client.clone());
    let get_user_roles_uc = GetUserRolesUseCase::new(keycloak_client.clone());
    let check_permission_uc = CheckPermissionUseCase::new(auth_domain_svc.clone());
    let record_audit_log_uc = RecordAuditLogUseCase::new(
        audit_repo.clone(), Arc::new(producer),
    );
    let search_audit_log_uc = SearchAuditLogUseCase::new(audit_repo.clone());

    let app_state = AppState {
        validate_token_uc: Arc::new(validate_token_uc),
        get_user_uc: Arc::new(get_user_uc),
        list_users_uc: Arc::new(list_users_uc),
        get_user_roles_uc: Arc::new(get_user_roles_uc),
        record_audit_log_uc: Arc::new(record_audit_log_uc),
        search_audit_log_uc: Arc::new(search_audit_log_uc),
        keycloak_client: keycloak_client.clone(),
        pool: pool.clone(),
    };

    // --- REST Server (axum) ---
    let rest_app = rest_handler::router(app_state);
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));

    let rest_handle = tokio::spawn(async move {
        info!("REST server starting on {}", rest_addr);
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        axum::serve(listener, rest_app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });

    // --- gRPC Server (tonic) ---
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], cfg.grpc.port));
    let grpc_service = grpc_handler::AuthServiceImpl::new(
        Arc::new(ValidateTokenUseCase::new(jwks_verifier, cfg.auth.jwt.clone())),
        Arc::new(GetUserUseCase::new(keycloak_client.clone())),
        Arc::new(ListUsersUseCase::new(keycloak_client.clone())),
        Arc::new(GetUserRolesUseCase::new(keycloak_client)),
        Arc::new(CheckPermissionUseCase::new(auth_domain_svc)),
    );

    let grpc_handle = tokio::spawn(async move {
        info!("gRPC server starting on {}", grpc_addr);
        TonicServer::builder()
            .add_service(grpc_handler::auth_service_server(grpc_service))
            .serve_with_shutdown(grpc_addr, shutdown_signal())
            .await
            .unwrap();
    });

    tokio::try_join!(rest_handle, grpc_handle)?;
    info!("servers exited");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
```

### ドメインモデル（Rust）

```rust
// src/domain/entity/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub attributes: HashMap<String, Vec<String>>,
}
```

```rust
// src/domain/entity/role.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub role: String,
    pub permission: String, // read, write, delete, admin
    pub resource: String,
}
```

```rust
// src/domain/entity/audit_log.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub action: String,
    pub resource: Option<String>,
    pub resource_id: Option<String>,
    pub result: String,
    #[sqlx(json)]
    pub detail: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### リポジトリトレイト（Rust）

```rust
// src/domain/repository/audit_log_repository.rs
use async_trait::async_trait;

use crate::domain::entity::AuditLog;

#[derive(Debug, Clone)]
pub struct AuditLogSearchParams {
    pub user_id: Option<String>,
    pub event_type: Option<String>,
    pub result: Option<String>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub page: i32,
    pub page_size: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()>;
    async fn search(
        &self,
        params: &AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<AuditLog>, i64)>;
}
```

### ユースケース（Rust）

```rust
// src/usecase/validate_token.rs
use std::sync::Arc;

use tracing::instrument;

use crate::infrastructure::auth::JwksVerifier;
use crate::infrastructure::config::JwtConfig;

pub struct ValidateTokenUseCase {
    verifier: Arc<JwksVerifier>,
    jwt_config: JwtConfig,
}

impl ValidateTokenUseCase {
    pub fn new(verifier: Arc<JwksVerifier>, jwt_config: JwtConfig) -> Self {
        Self { verifier, jwt_config }
    }

    #[instrument(skip(self, token), fields(service = "auth-server"))]
    pub async fn execute(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let claims = self.verifier.verify_token(token).await?;

        // issuer・audience の追加検証
        if claims.iss != self.jwt_config.issuer {
            return Err(AuthError::InvalidIssuer);
        }
        if claims.aud != self.jwt_config.audience {
            return Err(AuthError::InvalidAudience);
        }

        Ok(claims)
    }
}
```

### REST ハンドラー（Rust）

```rust
// src/adapter/handler/rest_handler.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::adapter::handler::error::ErrorResponse;
use crate::adapter::middleware;

pub fn router(state: AppState) -> Router {
    Router::new()
        // ヘルスチェック
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        // 公開エンドポイント
        .route("/api/v1/auth/token/validate", post(validate_token))
        // 認可付きエンドポイント
        .route(
            "/api/v1/auth/token/introspect",
            post(introspect_token).layer(middleware::require_permission("read", "auth_config")),
        )
        .route(
            "/api/v1/users",
            get(list_users).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id",
            get(get_user).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id/roles",
            get(get_user_roles).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/audit/logs",
            post(record_audit_log)
                .layer(middleware::require_permission("write", "audit_logs"))
                .get(search_audit_logs)
                .layer(middleware::require_permission("read", "audit_logs")),
        )
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let kc_ok = state.keycloak_client.healthy().await.is_ok();

    if db_ok && kc_ok {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "checks": {"database": "ok", "keycloak": "ok"}
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[derive(Deserialize)]
struct ValidateTokenRequest {
    token: String,
}

async fn validate_token(
    State(state): State<AppState>,
    Json(req): Json<ValidateTokenRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let claims = state
        .validate_token_uc
        .execute(&req.token)
        .await
        .map_err(|_| {
            ErrorResponse::unauthorized(
                "SYS_AUTH_TOKEN_INVALID",
                "トークンの検証に失敗しました",
            )
        })?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "claims": claims,
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = state.get_user_uc.execute(&id).await.map_err(|_| {
        ErrorResponse::not_found(
            "SYS_AUTH_USER_NOT_FOUND",
            "指定されたユーザーが見つかりません",
        )
    })?;

    Ok(Json(serde_json::to_value(user).unwrap()))
}

// ... 他のハンドラーも同様のパターンで実装
```

### gRPC ハンドラー（Rust）

```rust
// src/adapter/handler/grpc_handler.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("k1s0.system.auth.v1");
}

use proto::auth_service_server::{AuthService, AuthServiceServer};
use proto::*;

pub struct AuthServiceImpl {
    validate_token_uc: Arc<ValidateTokenUseCase>,
    get_user_uc: Arc<GetUserUseCase>,
    list_users_uc: Arc<ListUsersUseCase>,
    get_user_roles_uc: Arc<GetUserRolesUseCase>,
    check_permission_uc: Arc<CheckPermissionUseCase>,
}

impl AuthServiceImpl {
    pub fn new(
        validate_token_uc: Arc<ValidateTokenUseCase>,
        get_user_uc: Arc<GetUserUseCase>,
        list_users_uc: Arc<ListUsersUseCase>,
        get_user_roles_uc: Arc<GetUserRolesUseCase>,
        check_permission_uc: Arc<CheckPermissionUseCase>,
    ) -> Self {
        Self {
            validate_token_uc,
            get_user_uc,
            list_users_uc,
            get_user_roles_uc,
            check_permission_uc,
        }
    }
}

pub fn auth_service_server(svc: AuthServiceImpl) -> AuthServiceServer<AuthServiceImpl> {
    AuthServiceServer::new(svc)
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.validate_token_uc.execute(&req.token).await {
            Ok(claims) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                claims: Some(claims.into()),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                claims: None,
                error_message: e.to_string(),
            })),
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .get_user_uc
            .execute(&req.user_id)
            .await
            .map_err(|_| Status::not_found("user not found"))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user.into()),
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let (allowed, reason) = self
            .check_permission_uc
            .execute(&req.user_id, &req.permission, &req.resource, &req.roles)
            .await;

        Ok(Response::new(CheckPermissionResponse {
            allowed,
            reason,
        }))
    }

    // ListUsers, GetUserRoles も同様のパターンで実装
}
```

### Keycloak クライアント（Rust）

```rust
// src/adapter/gateway/keycloak_client.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::domain::entity::User;
use crate::infrastructure::config::OidcConfig;

pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
}

struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl KeycloakClient {
    pub fn new(cfg: &OidcConfig) -> Self {
        Self {
            base_url: extract_base_url(&cfg.discovery_url),
            realm: "k1s0".to_string(),
            client_id: cfg.client_id.clone(),
            client_secret: cfg.client_secret.clone(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            admin_token: Arc::new(RwLock::new(None)),
        }
    }

    #[instrument(skip(self), fields(service = "auth-server"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<User> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.base_url, self.realm, user_id
        );

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("user not found");
        }

        let user: User = resp.error_for_status()?.json().await?;
        Ok(user)
    }

    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.base_url, self.realm);
        let resp = self.http_client.get(&url).send().await?;
        resp.error_for_status()?;
        Ok(())
    }

    async fn get_admin_token(&self) -> anyhow::Result<String> {
        // Client Credentials Grant でトークンを取得（キャッシュ付き）
        let cache = self.admin_token.read().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }
        drop(cache);

        let mut cache = self.admin_token.write().await;
        // POST /realms/k1s0/protocol/openid-connect/token
        // ...（実装省略）
        Ok(cache.as_ref().unwrap().token.clone())
    }
}
```

---

## config.yaml

[config設計.md](config設計.md) のスキーマを拡張した認証サーバー固有の設定。

```yaml
# config/config.yaml
app:
  name: "auth-server"
  version: "0.1.0"
  tier: "system"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"

grpc:
  port: 50051
  max_recv_msg_size: 4194304  # 4MB

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""               # Vault パス: secret/data/k1s0/system/auth/database キー: password
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "auth-server.default"
  security_protocol: "PLAINTEXT"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: username
    password: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: password
  topics:
    publish:
      - "k1s0.system.auth.login.v1"
      - "k1s0.system.auth.token_validate.v1"
      - "k1s0.system.auth.permission_denied.v1"
    subscribe: []

observability:
  log:
    level: "debug"
    format: "json"
  trace:
    enabled: true
    endpoint: "jaeger.observability.svc.cluster.local:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"

auth:
  jwt:
    issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
    audience: "k1s0-api"
    public_key_path: ""
  oidc:
    discovery_url: "https://auth.k1s0.internal.example.com/realms/k1s0/.well-known/openid-configuration"
    client_id: "auth-server"
    client_secret: ""        # Vault パス: secret/data/k1s0/system/auth/oidc キー: client_secret
    redirect_uri: ""
    scopes: ["openid", "profile", "email"]
    jwks_uri: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"

# 認証サーバー固有設定
auth_server:
  # パーミッションキャッシュ
  permission_cache:
    ttl: "5m"                # ロール→パーミッション変換テーブルのキャッシュ TTL
    refresh_on_miss: true    # キャッシュミス時にバックグラウンドリフレッシュ
  # 監査ログ
  audit:
    kafka_enabled: true      # Kafka への非同期配信を有効化
    retention_days: 365      # DB 内の保持日数
  # Keycloak Admin API
  keycloak_admin:
    token_cache_ttl: "5m"    # Admin API トークンのキャッシュ TTL
```

### 設定の読み込み実装

```rust
// src/infrastructure/config/mod.rs
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: GrpcConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
    pub auth_server: AuthServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: String,
    pub write_timeout: String,
    pub shutdown_timeout: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthServerConfig {
    pub permission_cache: PermissionCacheConfig,
    pub audit: AuditServerConfig,
    pub keycloak_admin: KeycloakAdminConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionCacheConfig {
    pub ttl: String,
    pub refresh_on_miss: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditServerConfig {
    pub kafka_enabled: bool,
    pub retention_days: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakAdminConfig {
    pub token_cache_ttl: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        if self.auth.jwt.issuer.is_empty() {
            anyhow::bail!("auth.jwt.issuer is required");
        }
        if self.auth.oidc.jwks_uri.is_empty() {
            anyhow::bail!("auth.oidc.jwks_uri is required");
        }
        Ok(())
    }
}
```

---

## テスト構成（auth-server）

### JWKS TTL キャッシュテスト

`regions/system/server/rust/auth/tests/jwks_test.rs` に wiremock を使用した JWKS 取得テストを実装する。

| テスト名 | 内容 |
|---------|------|
| `test_jwks_fetch_from_mock_endpoint` | モックエンドポイントから JWKS を取得できる |
| `test_jwks_fetch_failure_returns_error` | JWKS 取得失敗時にエラーを返す |
| `test_jwks_cache_invalidation` | `invalidate_cache()` でキャッシュを無効化し再フェッチする |
| `test_jwks_empty_keys_response` | 空キーレスポンスでエラーを返す |
| `test_jwks_ttl_zero_always_refetches` | TTL=0 の場合は毎回 JWKS エンドポイントへフェッチする |
| `test_jwks_long_ttl_reuses_cached_keys` | TTL が十分長い場合は 2 回目以降キャッシュを再利用する（フェッチ 1 回のみ） |
| `test_jwks_concurrent_requests_no_panic` | 並行リクエスト中にパニックせず全リクエストがエラーを返す |

TTL 動作の詳細:

- `cache_ttl = Duration::ZERO`: キャッシュは常に期限切れ → 毎回フェッチ
- `cache_ttl = Duration::from_secs(3600)`: キャッシュが有効 → 再フェッチなし
- `invalidate_cache()` 呼び出し後は次のリクエストで強制フェッチ

---

## 関連ドキュメント

- [system-server設計.md](system-server設計.md) -- 概要・API 定義・アーキテクチャ
- [system-server-デプロイ設計.md](system-server-デプロイ設計.md) -- DB マイグレーション・テスト・Dockerfile・Helm values
- [config設計.md](config設計.md) -- config.yaml スキーマと環境別管理
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](コーディング規約.md) -- Linter・Formatter・命名規則
