# system-server 実装設計

> **ガイド**: 設計背景・実装例は [implementation.guide.md](./implementation.guide.md) を参照。

system-server（認証サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [system-server.md](../auth/server.md) を参照。

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

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# JWT
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

### build.rs

> build.rs パターンは [Rust共通実装.md](../_common/Rust共通実装.md#共通buildrs) を参照。proto パス: `api/proto/k1s0/system/auth/v1/auth.proto`

### src/main.rs

> 起動シーケンスは [Rust共通実装.md](../_common/Rust共通実装.md#共通mainrs) を参照。DI 実装例は [implementation.guide.md](./implementation.guide.md#サービス固有-dimainrs) を参照。

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

### ユースケース構造体（Rust）

```rust
// src/usecase/validate_token.rs
pub struct ValidateTokenUseCase {
    verifier: Arc<JwksVerifier>,
    jwt_config: JwtConfig,
}

impl ValidateTokenUseCase {
    pub fn new(verifier: Arc<JwksVerifier>, jwt_config: JwtConfig) -> Self {
        Self { verifier, jwt_config }
    }

    pub async fn execute(&self, token: &str) -> Result<TokenClaims, AuthError>;
}
```

> 完全な実装は [implementation.guide.md](./implementation.guide.md#validatetokenusecase) を参照。

### REST ルーティング（Rust）

| パス | メソッド | 認可 | ハンドラー |
|------|---------|------|-----------|
| `/healthz` | GET | なし | `healthz` |
| `/readyz` | GET | なし | `readyz` |
| `/metrics` | GET | なし | `metrics` |
| `/api/v1/auth/token/validate` | POST | なし | `validate_token` |
| `/api/v1/auth/token/introspect` | POST | `read:auth_config` | `introspect_token` |
| `/api/v1/users` | GET | `read:users` | `list_users` |
| `/api/v1/users/:id` | GET | `read:users` | `get_user` |
| `/api/v1/users/:id/roles` | GET | `read:users` | `get_user_roles` |
| `/api/v1/audit/logs` | POST | `write:audit_logs` | `record_audit_log` |
| `/api/v1/audit/logs` | GET | `read:audit_logs` | `search_audit_logs` |

> ハンドラー実装は [implementation.guide.md](./implementation.guide.md#rest-ハンドラー実装) を参照。

### gRPC サービス定義（Rust）

```rust
// src/adapter/handler/grpc_handler.rs
pub struct AuthServiceImpl {
    validate_token_uc: Arc<ValidateTokenUseCase>,
    get_user_uc: Arc<GetUserUseCase>,
    list_users_uc: Arc<ListUsersUseCase>,
    get_user_roles_uc: Arc<GetUserRolesUseCase>,
    check_permission_uc: Arc<CheckPermissionUseCase>,
}
```

> gRPC メソッド実装は [implementation.guide.md](./implementation.guide.md#grpc-ハンドラー実装) を参照。

### Keycloak クライアント定義（Rust）

```rust
// src/adapter/gateway/keycloak_client.rs
pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
}
```

> 実装は [implementation.guide.md](./implementation.guide.md#keycloak-クライアント実装) を参照。

---

## config.yaml

認証サーバー固有の設定セクション。共通セクション（app/server/database/kafka/observability）は [Rust共通実装.md](../_common/Rust共通実装.md#共通configyaml) を参照。

```yaml
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

### 設定構造体

```rust
// src/infrastructure/config/mod.rs
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
```

> `Config::load()` / `Config::validate()` 実装は [implementation.guide.md](./implementation.guide.md#設定読み込み実装) を参照。

---

## テスト構成（auth-server）

テストテーブルと TTL 動作仕様は [implementation.guide.md](./implementation.guide.md#テスト構成auth-server) を参照。

---

## 関連ドキュメント

- [system-server.md](../auth/server.md) -- 概要・API 定義・アーキテクチャ
- [system-server-deploy.md](deploy.md) -- DB マイグレーション・テスト・Dockerfile・Helm values
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
