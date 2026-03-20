# business-domain-master-server 実装仕様

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

#### MasterCategory (`domain/entity/master_category.rs`)

```rust
pub struct MasterCategory {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### MasterItem (`domain/entity/master_item.rs`)

```rust
pub struct MasterItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### MasterItemVersion (`domain/entity/master_item_version.rs`)

```rust
pub struct MasterItemVersion {
    pub id: Uuid,
    pub item_id: Uuid,
    pub version_number: i32,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

#### TenantMasterExtension (`domain/entity/tenant_master_extension.rs`)

```rust
pub struct TenantMasterExtension {
    pub id: Uuid,
    pub tenant_id: String,
    pub item_id: Uuid,
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### リポジトリトレイト

#### CategoryRepository (`domain/repository/category_repository.rs`)

```rust
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_by_code(&self, code: &str) -> Result<Option<MasterCategory>>;
    async fn find_all(&self, active_only: bool, pagination: &Pagination) -> Result<(Vec<MasterCategory>, i64)>;
    async fn create(&self, category: &MasterCategory) -> Result<MasterCategory>;
    async fn update(&self, category: &MasterCategory) -> Result<MasterCategory>;
    async fn delete(&self, code: &str) -> Result<bool>;
}
```

#### ItemRepository (`domain/repository/item_repository.rs`)

```rust
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_by_category_and_code(&self, category_id: Uuid, item_code: &str) -> Result<Option<MasterItem>>;
    async fn find_by_category(&self, category_id: Uuid, active_only: bool, pagination: &Pagination) -> Result<(Vec<MasterItem>, i64)>;
    async fn create(&self, item: &MasterItem) -> Result<MasterItem>;
    async fn update(&self, item: &MasterItem) -> Result<MasterItem>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}
```

#### ItemVersionRepository (`domain/repository/item_version_repository.rs`)

```rust
#[async_trait]
pub trait ItemVersionRepository: Send + Sync {
    async fn find_by_item(&self, item_id: Uuid, pagination: &Pagination) -> Result<(Vec<MasterItemVersion>, i64)>;
    async fn create(&self, version: &MasterItemVersion) -> Result<MasterItemVersion>;
    async fn next_version_number(&self, item_id: Uuid) -> Result<i32>;
}
```

#### TenantExtensionRepository (`domain/repository/tenant_extension_repository.rs`)

```rust
#[async_trait]
pub trait TenantExtensionRepository: Send + Sync {
    async fn find_by_tenant_and_item(&self, tenant_id: &str, item_id: Uuid) -> Result<Option<TenantMasterExtension>>;
    async fn find_by_tenant_and_category(&self, tenant_id: &str, category_id: Uuid) -> Result<Vec<TenantMasterExtension>>;
    async fn upsert(&self, extension: &TenantMasterExtension) -> Result<TenantMasterExtension>;
    async fn delete(&self, tenant_id: &str, item_id: Uuid) -> Result<bool>;
}
```

### ドメインサービス

#### ValidationService (`domain/service/validation_service.rs`)

カテゴリの `validation_schema` を使用して項目の `attributes` をバリデーションする。

```rust
pub struct ValidationService;

impl ValidationService {
    pub fn validate_attributes(
        validation_schema: &serde_json::Value,
        attributes: &serde_json::Value,
    ) -> Result<(), Vec<ValidationError>> {
        // jsonschema クレートで JSON Schema バリデーション
    }
}
```

#### ItemService (`domain/service/item_service.rs`)

親項目の循環参照チェック等のドメインロジック。

```rust
pub struct ItemService;

impl ItemService {
    pub fn check_circular_parent(
        item_id: Uuid,
        parent_item_id: Uuid,
        items: &[MasterItem],
    ) -> Result<()> {
        // 親を辿って循環参照がないか検証
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
    pub grpc_port: u16,   // 50061
    pub environment: String,
}
```

### 永続化

各リポジトリトレイトの実装。sqlx を使用して PostgreSQL の `domain_master` スキーマにアクセスする。

- `category_repo_impl.rs` — CategoryRepository 実装
- `item_repo_impl.rs` — ItemRepository 実装
- `item_version_repo_impl.rs` — ItemVersionRepository 実装
- `tenant_extension_repo_impl.rs` — TenantExtensionRepository 実装

### Kafka プロデューサー (`infrastructure/messaging/kafka_producer.rs`)

```rust
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_category_changed(&self, event: CategoryChangedEvent) -> Result<()>;
    async fn publish_item_changed(&self, event: ItemChangedEvent) -> Result<()>;
    async fn publish_tenant_extension_changed(&self, event: TenantExtensionChangedEvent) -> Result<()>;
}
```

---

## UseCase Layer

### ManageCategories (`usecase/manage_categories.rs`)

カテゴリの CRUD 操作。作成・更新・削除時に Kafka イベントを発行。

### ManageItems (`usecase/manage_items.rs`)

項目の CRUD 操作。更新時にバージョン自動記録、Kafka イベント発行。

```rust
pub struct ManageItemsUseCase {
    item_repo: Arc<dyn ItemRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    version_repo: Arc<dyn ItemVersionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ManageItemsUseCase {
    pub async fn update_item(&self, input: UpdateItemInput) -> Result<MasterItem> {
        // 1. 既存項目を取得
        // 2. カテゴリの validation_schema で attributes をバリデーション
        // 3. 項目を更新
        // 4. バージョンレコードを作成（before/after 差分）
        // 5. Kafka イベント発行
    }
}
```

### GetItemVersions (`usecase/get_item_versions.rs`)

項目の変更履歴を取得する。

### ManageTenantExtensions (`usecase/manage_tenant_extensions.rs`)

テナントカスタマイズの CRUD 操作とマージビューの生成。

---

## Adapter Layer

### REST ハンドラー

axum のルーターに登録。各ハンドラーは UseCase を呼び出し、Presenter でレスポンスに変換する。

### gRPC サービス (`adapter/grpc/tonic_service.rs`)

tonic で `DomainMasterService` を実装。REST ハンドラーと同じ UseCase を共有する。

### ミドルウェア

- `auth.rs` — JWT トークン検証
- `rbac.rs` — ロールベースアクセス制御（biz_auditor / biz_operator / biz_admin）

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
    let category_repo = Arc::new(PgCategoryRepository::new(pool.clone()));
    let item_repo = Arc::new(PgItemRepository::new(pool.clone()));
    let version_repo = Arc::new(PgItemVersionRepository::new(pool.clone()));
    let tenant_ext_repo = Arc::new(PgTenantExtensionRepository::new(pool.clone()));

    // Kafka プロデューサー
    let event_publisher = Arc::new(KafkaEventPublisher::new(&config.kafka)?);

    // ユースケース
    let manage_categories = Arc::new(ManageCategoriesUseCase::new(
        category_repo.clone(), event_publisher.clone(),
    ));
    let manage_items = Arc::new(ManageItemsUseCase::new(
        item_repo.clone(), category_repo.clone(),
        version_repo.clone(), event_publisher.clone(),
    ));
    let get_versions = Arc::new(GetItemVersionsUseCase::new(version_repo.clone()));
    let manage_tenant_ext = Arc::new(ManageTenantExtensionsUseCase::new(
        tenant_ext_repo.clone(), item_repo.clone(), event_publisher.clone(),
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
    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Duplicate category code: {0}")]
    DuplicateCategory(String),

    #[error("Duplicate item code: {0}")]
    DuplicateItem(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Circular parent reference detected")]
    CircularParent,

    #[error("Tenant extension not found")]
    TenantExtensionNotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
```

`AppError` → HTTP ステータスコード + `BIZ_DOMAINMASTER_*` エラーコードへの変換は `adapter/handler/error.rs` で実装する。

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
    async fn test_update_item_creates_version() {
        let mut mock_item_repo = MockItemRepository::new();
        let mut mock_version_repo = MockItemVersionRepository::new();
        // ... mock 設定
        // バージョンレコードが作成されることを検証
        mock_version_repo
            .expect_create()
            .times(1)
            .returning(|v| Ok(v.clone()));
    }
}
```

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
