# system-config-server 実装ガイド

> **仕様**: テーブル定義・APIスキーマは [implementation.md](./implementation.md) を参照。

---

## src/main.rs サービス固有の DI

> 起動シーケンスは [Rust共通実装.md](../_common/Rust共通実装.md#共通mainrs) を参照。以下はサービス固有の DI:

```rust
    // --- Cache ---
    let config_cache = Arc::new(ConfigCache::new(
        cfg.config_server.cache.ttl_seconds,
        cfg.config_server.cache.max_entries,
    ));

    // --- DI ---
    let config_repo = Arc::new(persistence::ConfigRepositoryImpl::new(pool.clone()));
    let change_log_repo = Arc::new(persistence::ConfigChangeLogRepositoryImpl::new(pool.clone()));
    let config_domain_svc = Arc::new(ConfigDomainService::new());

    let get_config_uc = GetConfigUseCase::new(config_repo.clone(), config_cache.clone());
    let list_configs_uc = ListConfigsUseCase::new(config_repo.clone());
    let update_config_uc = UpdateConfigUseCase::new(
        config_repo.clone(),
        change_log_repo.clone(),
        config_domain_svc.clone(),
        config_cache.clone(),
        producer.clone(),
    );
    let delete_config_uc = DeleteConfigUseCase::new(
        config_repo.clone(),
        change_log_repo.clone(),
        config_cache.clone(),
        producer.clone(),
    );
    let get_service_config_uc = GetServiceConfigUseCase::new(
        config_repo.clone(), config_cache.clone(),
    );

    let app_state = AppState {
        get_config_uc: Arc::new(get_config_uc),
        list_configs_uc: Arc::new(list_configs_uc),
        update_config_uc: Arc::new(update_config_uc),
        delete_config_uc: Arc::new(delete_config_uc),
        get_service_config_uc: Arc::new(get_service_config_uc),
        pool: pool.clone(),
        producer: producer.clone(),
    };
```

---

## ドメインモデル実装（Rust）

```rust
// src/domain/entity/config_entry.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigEntry {
    pub id: Uuid,
    pub namespace: String,
    pub key: String,
    #[sqlx(json)]
    pub value: serde_json::Value,
    pub version: i32,
    pub description: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// src/domain/entity/config_change_log.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigChangeLog {
    pub id: Uuid,
    pub config_entry_id: Uuid,
    pub namespace: String,
    pub key: String,
    #[sqlx(json)]
    pub old_value: Option<serde_json::Value>,
    #[sqlx(json)]
    pub new_value: Option<serde_json::Value>,
    pub old_version: i32,
    pub new_version: i32,
    pub change_type: String, // CREATED, UPDATED, DELETED
    pub changed_by: String,
    pub changed_at: DateTime<Utc>,
}
```

---

## リポジトリトレイト実装（Rust）

```rust
// src/domain/repository/config_repository.rs
use async_trait::async_trait;

use crate::domain::entity::ConfigEntry;

#[derive(Debug, Clone)]
pub struct ListParams {
    pub search: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>>;

    async fn list_by_namespace(
        &self,
        namespace: &str,
        params: &ListParams,
    ) -> anyhow::Result<(Vec<ConfigEntry>, i64)>;

    async fn list_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<Vec<ConfigEntry>>;

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<()>;
    async fn update(&self, entry: &ConfigEntry) -> anyhow::Result<()>;
    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<()>;
}
```

```rust
// src/domain/repository/config_change_log_repository.rs
use async_trait::async_trait;

use crate::domain::entity::ConfigChangeLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigChangeLogRepository: Send + Sync {
    async fn create(&self, log: &ConfigChangeLog) -> anyhow::Result<()>;
    async fn list_by_config_entry_id(
        &self,
        config_entry_id: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<ConfigChangeLog>, i64)>;
}
```

---

## ユースケース実装（Rust）

### GetConfigUseCase

```rust
// src/usecase/get_config.rs
use std::sync::Arc;

use tracing::instrument;

use crate::domain::entity::ConfigEntry;
use crate::domain::repository::ConfigRepository;
use crate::infrastructure::cache::ConfigCache;

pub struct GetConfigUseCase {
    repo: Arc<dyn ConfigRepository>,
    cache: Arc<ConfigCache>,
}

impl GetConfigUseCase {
    pub fn new(repo: Arc<dyn ConfigRepository>, cache: Arc<ConfigCache>) -> Self {
        Self { repo, cache }
    }

    #[instrument(skip(self), fields(service = "config-server"))]
    pub async fn execute(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<ConfigEntry, ConfigError> {
        // キャッシュから取得を試みる
        let cache_key = format!("{}:{}", namespace, key);
        if let Some(entry) = self.cache.get(&cache_key).await {
            return Ok(entry);
        }

        // DB から取得
        let entry = self
            .repo
            .find_by_namespace_and_key(namespace, key)
            .await
            .map_err(|_| ConfigError::InternalError)?
            .ok_or(ConfigError::KeyNotFound)?;

        // キャッシュに格納
        self.cache.set(&cache_key, &entry).await;

        Ok(entry)
    }
}
```

### UpdateConfigUseCase

```rust
// src/usecase/update_config.rs
use std::sync::Arc;

use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::entity::{ConfigChangeLog, ConfigEntry};
use crate::domain::repository::{ConfigChangeLogRepository, ConfigRepository};
use crate::domain::service::ConfigDomainService;
use crate::infrastructure::cache::ConfigCache;
use crate::infrastructure::messaging::KafkaProducer;

pub struct UpdateConfigUseCase {
    repo: Arc<dyn ConfigRepository>,
    change_log_repo: Arc<dyn ConfigChangeLogRepository>,
    domain_svc: Arc<ConfigDomainService>,
    cache: Arc<ConfigCache>,
    producer: Arc<KafkaProducer>,
}

impl UpdateConfigUseCase {
    pub fn new(
        repo: Arc<dyn ConfigRepository>,
        change_log_repo: Arc<dyn ConfigChangeLogRepository>,
        domain_svc: Arc<ConfigDomainService>,
        cache: Arc<ConfigCache>,
        producer: Arc<KafkaProducer>,
    ) -> Self {
        Self {
            repo,
            change_log_repo,
            domain_svc,
            cache,
            producer,
        }
    }

    #[instrument(skip(self, input), fields(service = "config-server"))]
    pub async fn execute(
        &self,
        input: UpdateConfigInput,
    ) -> Result<ConfigEntry, ConfigError> {
        // namespace バリデーション
        self.domain_svc
            .validate_namespace(&input.namespace)
            .map_err(|_| ConfigError::InvalidNamespace)?;

        // 現在の設定値を取得
        let mut current = self
            .repo
            .find_by_namespace_and_key(&input.namespace, &input.key)
            .await
            .map_err(|_| ConfigError::InternalError)?
            .ok_or(ConfigError::KeyNotFound)?;

        // バージョン検証（楽観的排他制御）
        self.domain_svc
            .validate_version(current.version, input.version)
            .map_err(|_| ConfigError::VersionConflict)?;

        // 更新
        let old_value = current.value.clone();
        current.value = input.value;
        current.version += 1;
        current.description = input.description;
        current.updated_by = input.updated_by.clone();
        current.updated_at = Utc::now();

        self.repo
            .update(&current)
            .await
            .map_err(|_| ConfigError::InternalError)?;

        // 変更ログ記録
        let change_log = ConfigChangeLog {
            id: Uuid::new_v4(),
            config_entry_id: current.id,
            namespace: current.namespace.clone(),
            key: current.key.clone(),
            old_value: Some(old_value),
            new_value: Some(current.value.clone()),
            old_version: input.version,
            new_version: current.version,
            change_type: "UPDATED".to_string(),
            changed_by: input.updated_by,
            changed_at: current.updated_at,
        };
        let _ = self.change_log_repo.create(&change_log).await;

        // キャッシュ無効化
        let cache_key = format!("{}:{}", current.namespace, current.key);
        self.cache.invalidate(&cache_key).await;

        // Kafka 通知
        self.producer.publish_config_changed(&change_log).await;

        Ok(current)
    }
}

pub struct UpdateConfigInput {
    pub namespace: String,
    pub key: String,
    pub value: serde_json::Value,
    pub version: i32,
    pub description: String,
    pub updated_by: String,
}
```

---

## REST ハンドラー実装（Rust）

```rust
// src/adapter/handler/rest_handler.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, put},
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
        // 設定値管理
        .route(
            "/api/v1/config/:namespace/:key",
            get(get_config)
                .layer(middleware::require_permission("read", "config"))
                .put(update_config)
                .layer(middleware::require_permission("write", "config"))
                .delete(delete_config)
                .layer(middleware::require_permission("admin", "config")),
        )
        .route(
            "/api/v1/config/:namespace",
            get(list_configs).layer(middleware::require_permission("read", "config")),
        )
        // 設定スキーマ管理
        .route(
            "/api/v1/config-schema",
            get(list_config_schemas)
                .layer(middleware::require_permission("read", "config-schema"))
                .post(create_config_schema)
                .layer(middleware::require_permission("write", "config-schema")),
        )
        .route(
            "/api/v1/config-schema/:name",
            get(get_config_schema)
                .layer(middleware::require_permission("read", "config-schema"))
                .put(update_config_schema)
                .layer(middleware::require_permission("write", "config-schema")),
        )
        // サービス向け設定一括取得
        .route(
            "/api/v1/config/services/:service_name",
            get(get_service_config).layer(middleware::require_bearer_token()),
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

    let kafka_ok = state.producer.healthy().await.is_ok();

    if db_ok && kafka_ok {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "checks": {"database": "ok", "kafka": "ok"}
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn get_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let entry = state
        .get_config_uc
        .execute(&namespace, &key)
        .await
        .map_err(|_| {
            ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            )
        })?;

    Ok(Json(serde_json::to_value(entry).unwrap()))
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    value: serde_json::Value,
    version: i32,
    description: Option<String>,
}

async fn update_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let updated_by = middleware::get_user_email_from_context();

    let input = UpdateConfigInput {
        namespace,
        key,
        value: req.value,
        version: req.version,
        description: req.description.unwrap_or_default(),
        updated_by,
    };

    let entry = state
        .update_config_uc
        .execute(input)
        .await
        .map_err(|e| match e {
            ConfigError::VersionConflict => ErrorResponse::conflict(
                "SYS_CONFIG_VERSION_CONFLICT",
                "設定値が他のユーザーによって更新されています。最新のバージョンを取得してください",
            ),
            ConfigError::KeyNotFound => ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            ),
            _ => ErrorResponse::internal(
                "SYS_CONFIG_UPDATE_FAILED",
                "設定値の更新に失敗しました",
            ),
        })?;

    Ok(Json(serde_json::to_value(entry).unwrap()))
}

async fn delete_config(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Result<StatusCode, ErrorResponse> {
    let deleted_by = middleware::get_user_email_from_context();

    state
        .delete_config_uc
        .execute(&namespace, &key, &deleted_by)
        .await
        .map_err(|_| {
            ErrorResponse::not_found(
                "SYS_CONFIG_KEY_NOT_FOUND",
                "指定された設定キーが見つかりません",
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// config-schema ハンドラー（同様のパターンで実装）
async fn list_config_schemas(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // ... list_config_schemas_uc.execute().await
    todo!()
}

async fn create_config_schema(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // ... create_config_schema_uc.execute(req).await
    todo!()
}

async fn get_config_schema(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // ... get_config_schema_uc.execute(&name).await
    todo!()
}

async fn update_config_schema(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    // ... update_config_schema_uc.execute(&name, req).await
    todo!()
}

// ... 他のハンドラーも同様のパターンで実装
```

---

## gRPC ハンドラー実装（Rust）

```rust
// src/adapter/handler/grpc_handler.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("k1s0.system.config.v1");
}

use proto::config_service_server::{ConfigService, ConfigServiceServer};
use proto::*;

pub struct ConfigServiceImpl {
    get_config_uc: Arc<GetConfigUseCase>,
    list_configs_uc: Arc<ListConfigsUseCase>,
    get_service_config_uc: Arc<GetServiceConfigUseCase>,
}

impl ConfigServiceImpl {
    pub fn new(
        get_config_uc: Arc<GetConfigUseCase>,
        list_configs_uc: Arc<ListConfigsUseCase>,
        get_service_config_uc: Arc<GetServiceConfigUseCase>,
    ) -> Self {
        Self {
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
        }
    }
}

pub fn config_service_server(svc: ConfigServiceImpl) -> ConfigServiceServer<ConfigServiceImpl> {
    ConfigServiceServer::new(svc)
}

#[tonic::async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let req = request.into_inner();

        let entry = self
            .get_config_uc
            .execute(&req.namespace, &req.key)
            .await
            .map_err(|_| Status::not_found("config entry not found"))?;

        Ok(Response::new(GetConfigResponse {
            entry: Some(entry.into()),
        }))
    }

    async fn list_configs(
        &self,
        request: Request<ListConfigsRequest>,
    ) -> Result<Response<ListConfigsResponse>, Status> {
        let req = request.into_inner();

        let (entries, total) = self
            .list_configs_uc
            .execute(
                &req.namespace,
                ListConfigsInput {
                    search: if req.search.is_empty() { None } else { Some(req.search) },
                    page: req.pagination.as_ref().map(|p| p.page).unwrap_or(1),
                    page_size: req.pagination.as_ref().map(|p| p.page_size).unwrap_or(20),
                },
            )
            .await
            .map_err(|_| Status::internal("failed to list configs"))?;

        let proto_entries: Vec<proto::ConfigEntry> =
            entries.into_iter().map(|e| e.into()).collect();

        Ok(Response::new(ListConfigsResponse {
            entries: proto_entries,
            pagination: Some(proto::PaginationResult {
                total_count: total as i32,
            }),
        }))
    }

    async fn get_service_config(
        &self,
        request: Request<GetServiceConfigRequest>,
    ) -> Result<Response<GetServiceConfigResponse>, Status> {
        let req = request.into_inner();

        let entries = self
            .get_service_config_uc
            .execute(&req.service_name)
            .await
            .map_err(|_| Status::not_found("service config not found"))?;

        let proto_entries: Vec<proto::ConfigEntry> =
            entries.into_iter().map(|e| e.into()).collect();

        Ok(Response::new(GetServiceConfigResponse {
            service_name: req.service_name,
            entries: proto_entries,
        }))
    }

    type WatchConfigStream = tokio_stream::wrappers::ReceiverStream<
        Result<ConfigChangeEvent, Status>,
    >;

    async fn watch_config(
        &self,
        request: Request<WatchConfigRequest>,
    ) -> Result<Response<Self::WatchConfigStream>, Status> {
        let req = request.into_inner();
        let namespaces = req.namespaces;

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Kafka Consumer からのイベントをストリームとして配信
        // ...（実装省略）

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
```

---

## config.yaml サービス固有セクション

> 共通セクション（app/server/database/kafka/observability）は [Rust共通実装.md](../_common/Rust共通実装.md#共通configyaml) を参照。サービス固有セクション:

```yaml
# 設定管理サーバー固有設定
config_server:
  # インメモリキャッシュ
  cache:
    ttl: "60s"               # キャッシュの TTL（デフォルト 60 秒）
    max_entries: 10000        # キャッシュの最大エントリ数
    refresh_on_miss: true     # キャッシュミス時にバックグラウンドリフレッシュ
  # 監査ログ
  audit:
    kafka_enabled: true       # Kafka への非同期配信を有効化
    retention_days: 365       # DB 内の保持日数
  # namespace バリデーション
  namespace:
    allowed_tiers:
      - "system"
      - "business"
      - "service"
    max_depth: 4              # namespace の最大階層数
```

### 設定の読み込み実装（Go）

```go
// internal/infra/config/config.go
package config

import (
    "fmt"
    "os"
    "time"

    "github.com/go-playground/validator/v10"
    "gopkg.in/yaml.v3"
)

type Config struct {
    App           AppConfig           `yaml:"app"`
    Server        ServerConfig        `yaml:"server"`
    GRPC          GRPCConfig          `yaml:"grpc"`
    Database      DatabaseConfig      `yaml:"database"`
    Kafka         KafkaConfig         `yaml:"kafka"`
    Observability ObservabilityConfig `yaml:"observability"`
    ConfigServer  ConfigServerConfig  `yaml:"config_server"`
}

type ServerConfig struct {
    Host            string        `yaml:"host"`
    Port            int           `yaml:"port" validate:"required,min=1,max=65535"`
    ReadTimeout     time.Duration `yaml:"read_timeout"`
    WriteTimeout    time.Duration `yaml:"write_timeout"`
    ShutdownTimeout time.Duration `yaml:"shutdown_timeout"`
}

type ConfigServerConfig struct {
    Cache     CacheConfig     `yaml:"cache"`
    Audit     AuditConfig     `yaml:"audit"`
    Namespace NamespaceConfig `yaml:"namespace"`
}

type CacheConfig struct {
    TTL           string `yaml:"ttl"`
    MaxEntries    int    `yaml:"max_entries"`
    RefreshOnMiss bool   `yaml:"refresh_on_miss"`
}

type AuditConfig struct {
    KafkaEnabled  bool `yaml:"kafka_enabled"`
    RetentionDays int  `yaml:"retention_days"`
}

type NamespaceConfig struct {
    AllowedTiers []string `yaml:"allowed_tiers"`
    MaxDepth     int      `yaml:"max_depth"`
}

func Load(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("failed to read config: %w", err)
    }
    var cfg Config
    if err := yaml.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("failed to parse config: %w", err)
    }
    return &cfg, nil
}

func (c *Config) Validate() error {
    validate := validator.New()
    return validate.Struct(c)
}
```

### 設定の読み込み実装（Rust）

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
    pub config_server: ConfigServerConfig,
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
pub struct ConfigServerConfig {
    pub cache: CacheConfig,
    pub audit: AuditServerConfig,
    pub namespace: NamespaceConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub ttl: String,
    pub max_entries: u64,
    pub refresh_on_miss: bool,

    #[serde(skip)]
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditServerConfig {
    pub kafka_enabled: bool,
    pub retention_days: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceConfig {
    pub allowed_tiers: Vec<String>,
    pub max_depth: usize,
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
        if self.config_server.cache.max_entries == 0 {
            anyhow::bail!("config_server.cache.max_entries must be > 0");
        }
        if self.config_server.namespace.allowed_tiers.is_empty() {
            anyhow::bail!("config_server.namespace.allowed_tiers must not be empty");
        }
        Ok(())
    }
}
```

---

## WatchConfig gRPC Stream テスト

`regions/system/server/rust/config/tests/grpc_stream_test.rs` に外部インテグレーションテストを実装する。

`WatchConfigStreamHandler` のインライン単体テスト（`watch_stream.rs` 内、7件）に加え、複数クライアントシナリオを外部テストで検証する:

| テスト名 | 内容 |
|---------|------|
| `test_watch_config_multiple_clients_receive_same_event` | 複数クライアントが同一の設定変更イベントを受信する |
| `test_watch_config_multiple_clients_independent_namespace_filters` | クライアントごとに独立した namespace フィルタが適用される |
| `test_watch_config_subscriber_receives_only_post_subscription_events` | サブスクライブ後に発行されたイベントのみ受信する（サブスクライブ前は受信しない） |

`ConfigChangedEvent` の JSON スキーマ例:

```json
{
  "namespace": "system",
  "key": "feature.flags.new_ui",
  "old_value": false,
  "new_value": true,
  "old_version": 3,
  "new_version": 4,
  "change_type": "UPDATED"
}
```

`change_type` の値: `CREATED`（新規作成）/ `UPDATED`（更新）/ `DELETED`（削除）

---

## 関連ドキュメント

- [system-config-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-config-server-implementation.md](implementation.md) -- ディレクトリ構成・Cargo.toml
- [system-config-server-deploy.md](deploy.md) -- キャッシュ戦略・DB マイグレーション・テスト・Dockerfile・Helm values
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
