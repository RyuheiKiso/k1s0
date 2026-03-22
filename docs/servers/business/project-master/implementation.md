# business-project-master-server 実装仕様

business-project-master-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

## レイヤー構成

| レイヤー | ディレクトリ | 責務 |
| --- | --- | --- |
| domain | `src/domain/` | エンティティ・リポジトリトレイト・ドメインサービス |
| infrastructure | `src/infrastructure/` | 設定・永続化実装・Kafka プロデューサー |
| usecase | `src/usecase/` | アプリケーションユースケース |
| adapter | `src/adapter/` | REST ハンドラー・gRPC サービス・ミドルウェア・プレゼンター |

### レイヤー依存関係

```
adapter → usecase → domain ← infrastructure
```

- `domain` は外部に依存しない（Pure Rust）
- `infrastructure` は `domain` のリポジトリトレイトを実装
- `usecase` は `domain` のトレイトに依存（具象実装に依存しない）
- `adapter` は `usecase` を呼び出し、HTTP/gRPC レスポンスに変換

---

## Domain Layer

### エンティティ

#### ProjectType (`domain/entity/project_type.rs`)

```rust
pub struct ProjectType {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub version: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### StatusDefinition (`domain/entity/status_definition.rs`)

```rust
pub struct StatusDefinition {
    pub id: Uuid,
    pub project_type_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_terminal: bool,
    pub sort_order: i32,
    pub version: i32,
    pub tenant_id: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### StatusDefinitionVersion (`domain/entity/status_definition_version.rs`)

```rust
pub struct StatusDefinitionVersion {
    pub id: Uuid,
    pub status_definition_id: Uuid,
    pub version_number: i32,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

#### TenantProjectExtension (`domain/entity/tenant_project_extension.rs`)

```rust
pub struct TenantProjectExtension {
    pub id: Uuid,
    pub tenant_id: String,
    pub project_type_id: Uuid,
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### リポジトリトレイト

#### ProjectTypeRepository (`domain/repository/project_type_repository.rs`)

```rust
#[async_trait]
pub trait ProjectTypeRepository: Send + Sync {
    async fn find_by_code(&self, code: &str) -> Result<Option<ProjectType>>;
    async fn find_all(&self, active_only: bool, pagination: &Pagination) -> Result<(Vec<ProjectType>, i64)>;
    async fn create(&self, project_type: &ProjectType) -> Result<ProjectType>;
    async fn update(&self, project_type: &ProjectType) -> Result<ProjectType>;
    async fn delete(&self, code: &str) -> Result<bool>;
}
```

#### StatusDefinitionRepository (`domain/repository/status_definition_repository.rs`)

```rust
#[async_trait]
pub trait StatusDefinitionRepository: Send + Sync {
    async fn find_by_project_type_and_code(
        &self, project_type_id: Uuid, status_code: &str,
    ) -> Result<Option<StatusDefinition>>;
    async fn find_by_project_type(
        &self, project_type_id: Uuid, pagination: &Pagination,
    ) -> Result<(Vec<StatusDefinition>, i64)>;
    async fn create(&self, status: &StatusDefinition) -> Result<StatusDefinition>;
    async fn update(&self, status: &StatusDefinition) -> Result<StatusDefinition>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}
```

#### StatusDefinitionVersionRepository (`domain/repository/status_definition_version_repository.rs`)

```rust
#[async_trait]
pub trait StatusDefinitionVersionRepository: Send + Sync {
    async fn find_by_status_definition(
        &self, status_definition_id: Uuid, pagination: &Pagination,
    ) -> Result<(Vec<StatusDefinitionVersion>, i64)>;
    async fn create(&self, version: &StatusDefinitionVersion) -> Result<StatusDefinitionVersion>;
    async fn next_version_number(&self, status_definition_id: Uuid) -> Result<i32>;
}
```

#### TenantExtensionRepository (`domain/repository/tenant_extension_repository.rs`)

```rust
#[async_trait]
pub trait TenantExtensionRepository: Send + Sync {
    async fn find_by_tenant_and_type(
        &self, tenant_id: &str, project_type_id: Uuid,
    ) -> Result<Option<TenantProjectExtension>>;
    async fn upsert(&self, extension: &TenantProjectExtension) -> Result<TenantProjectExtension>;
    async fn delete(&self, tenant_id: &str, project_type_id: Uuid) -> Result<bool>;
}
```

### ドメインサービス

#### ProjectTypeService (`domain/service/project_type_service.rs`)

プロジェクトタイプ固有のドメインロジックを提供する。

```rust
pub struct ProjectTypeService;

impl ProjectTypeService {
    /// ステータス定義の終端ステータス整合性を検証する。
    pub fn validate_terminal_statuses(
        statuses: &[StatusDefinition],
    ) -> Result<(), Vec<ValidationError>> {
        // 少なくとも1つの終端ステータスが存在することを検証
    }
}
```

---

## Infrastructure Layer

### 設定 (`infrastructure/config/app_config.rs`)

```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub auth: AuthConfig,
}

pub struct ServerConfig {
    pub port: u16,        // 8210
    pub grpc_port: u16,   // 9210
    pub environment: String,
}
```

### 永続化

各リポジトリトレイトの実装。sqlx を使用して PostgreSQL の `project_master` スキーマにアクセスする。

- `project_type_repo_impl.rs` — ProjectTypeRepository 実装
- `status_definition_repo_impl.rs` — StatusDefinitionRepository 実装
- `status_definition_version_repo_impl.rs` — StatusDefinitionVersionRepository 実装
- `tenant_extension_repo_impl.rs` — TenantExtensionRepository 実装

### Kafka プロデューサー (`infrastructure/messaging/kafka_producer.rs`)

Outbox pattern を採用し、ドメインイベントを同一トランザクション内で outbox_events テーブルに書き込む。バックグラウンドワーカーが未配信イベントを Kafka に配信する。

```rust
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_project_type_changed(&self, event: ProjectTypeChangedEvent) -> Result<()>;
    async fn publish_status_definition_changed(
        &self, event: StatusDefinitionChangedEvent,
    ) -> Result<()>;
}
```

---

## UseCase Layer

### ManageProjectTypes (`usecase/manage_project_types.rs`)

プロジェクトタイプの CRUD 操作。作成・更新・削除時に Kafka イベントを発行（Outbox pattern）。楽観的ロック（version フィールド）による同時更新競合を検知する。

```rust
pub struct ManageProjectTypesUseCase {
    project_type_repo: Arc<dyn ProjectTypeRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ManageProjectTypesUseCase {
    pub async fn update_project_type(&self, input: UpdateProjectTypeInput) -> Result<ProjectType> {
        // 1. 既存プロジェクトタイプを取得
        // 2. version フィールドで楽観的ロックチェック
        // 3. プロジェクトタイプを更新
        // 4. Outbox にイベントを書き込み（同一トランザクション）
    }
}
```

### ManageStatusDefinitions (`usecase/manage_status_definitions.rs`)

ステータス定義の CRUD 操作。更新時にバージョン自動記録、Outbox pattern でイベント発行。

```rust
pub struct ManageStatusDefinitionsUseCase {
    status_definition_repo: Arc<dyn StatusDefinitionRepository>,
    project_type_repo: Arc<dyn ProjectTypeRepository>,
    version_repo: Arc<dyn StatusDefinitionVersionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ManageStatusDefinitionsUseCase {
    pub async fn update_status_definition(
        &self, input: UpdateStatusDefinitionInput,
    ) -> Result<StatusDefinition> {
        // 1. 既存ステータス定義を取得
        // 2. ステータス定義を更新
        // 3. バージョンレコードを作成（before/after 差分）
        // 4. Outbox にイベントを書き込み（同一トランザクション）
    }
}
```

### GetStatusDefinitionVersions (`usecase/get_status_definition_versions.rs`)

ステータス定義の変更履歴を取得する。

### ManageTenantExtensions (`usecase/manage_tenant_extensions.rs`)

テナント拡張の作成・更新（upsert）・削除操作。

---

## Adapter Layer

### REST ハンドラー

axum のルーターに登録。各ハンドラーは UseCase を呼び出し、Presenter でレスポンスに変換する。

### gRPC サービス (`adapter/grpc/tonic_service.rs`)

tonic で `ProjectMasterService` を実装。REST ハンドラーと同じ UseCase を共有する。

### ミドルウェア

- `auth.rs` — JWT トークン検証
- `rbac.rs` — ロールベースアクセス制御（project_type / status_definition / tenant_extension 各リソース）

---

## DI (Dependency Injection)

`main.rs` で Arc 経由の手動 DI を行う。

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 設定読み込み
    let config = AppConfig::load()?;

    // DB 接続プール
    let pool = PgPool::connect(&config.database.url).await?;

    // リポジトリ実装
    let project_type_repo = Arc::new(PgProjectTypeRepository::new(pool.clone()));
    let status_definition_repo = Arc::new(PgStatusDefinitionRepository::new(pool.clone()));
    let version_repo = Arc::new(PgStatusDefinitionVersionRepository::new(pool.clone()));
    let tenant_ext_repo = Arc::new(PgTenantExtensionRepository::new(pool.clone()));

    // Kafka プロデューサー（Outbox pattern）
    let event_publisher = Arc::new(KafkaEventPublisher::new(&config.kafka, pool.clone())?);

    // ユースケース
    let manage_project_types = Arc::new(ManageProjectTypesUseCase::new(
        project_type_repo.clone(), event_publisher.clone(),
    ));
    let manage_status_definitions = Arc::new(ManageStatusDefinitionsUseCase::new(
        status_definition_repo.clone(), project_type_repo.clone(),
        version_repo.clone(), event_publisher.clone(),
    ));
    let get_versions = Arc::new(GetStatusDefinitionVersionsUseCase::new(version_repo.clone()));
    let manage_tenant_ext = Arc::new(ManageTenantExtensionsUseCase::new(
        tenant_ext_repo.clone(), project_type_repo.clone(),
    ));

    // axum ルーター + tonic サービス起動
    // ...
}
```

---

## エラー処理

`anyhow::Result` をベースとし、`AppError` に変換して HTTP/gRPC レスポンスを生成する。

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Project type not found: {0}")]
    ProjectTypeNotFound(String),

    #[error("Status definition not found: {0}")]
    StatusDefinitionNotFound(String),

    #[error("Duplicate project type code: {0}")]
    DuplicateProjectType(String),

    #[error("Duplicate status code: {0}")]
    DuplicateStatus(String),

    #[error("Tenant extension not found")]
    TenantExtensionNotFound,

    #[error("Version conflict for: {0}")]
    VersionConflict(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
```

`AppError` → HTTP ステータスコード + `BIZ_PROJECTMASTER_*` エラーコードへの変換は `adapter/handler/error.rs` で実装する。

---

## テスト戦略

| テスト種別 | ツール | 対象 |
| --- | --- | --- |
| 単体テスト | mockall | ドメインサービス・ユースケース |
| 統合テスト | testcontainers | リポジトリ実装（PostgreSQL） |
| API テスト | tower::ServiceExt | REST ハンドラー |

### mockall によるユースケーステスト例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_update_status_definition_creates_version() {
        let mut mock_status_repo = MockStatusDefinitionRepository::new();
        let mut mock_version_repo = MockStatusDefinitionVersionRepository::new();
        // バージョンレコードが作成されることを検証
        mock_version_repo
            .expect_create()
            .times(1)
            .returning(|v| Ok(v.clone()));
    }
}
```

---

## Cargo.toml

```toml
[package]
name = "k1s0-project-master-server"
version = "0.1.0"
edition = "2021"

[lib]
name = "k1s0_project_master_server"
path = "src/lib.rs"

[[bin]]
name = "k1s0-project-master-server"
path = "src/main.rs"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"

# Logging / Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Observability (共通テンプレート準拠)
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.28"
prometheus = "0.13"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Telemetry library
k1s0-telemetry = { path = "../../../../../system/library/rust/telemetry", features = ["full"] }

# Auth library
k1s0-auth = { path = "../../../../../system/library/rust/auth" }

# Server common (error codes, auth middleware, RBAC, gRPC auth)
k1s0-server-common = { path = "../../../../../system/library/rust/server-common", features = ["axum", "grpc-auth"] }

# OpenAPI
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# Kafka
rdkafka = { version = "0.37", features = ["cmake-build"] }

[build-dependencies]
tonic-build = "0.12"

[features]
db-tests = []

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
testcontainers = "0.23"
axum-test = "17"
```

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
