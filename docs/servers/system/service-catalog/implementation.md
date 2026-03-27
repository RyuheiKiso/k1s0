# system-service-catalog-server 実装設計

system-service-catalog-server（サービスカタログサーバー）の Rust 実装仕様。概要・API 定義は [server.md](server.md) を参照。

---

## Rust 実装 (regions/system/server/rust/service-catalog/)

### ディレクトリ構成

```
regions/system/server/rust/service-catalog/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── service.rs               # Service エンティティ
│   │   │   ├── team.rs                  # Team エンティティ
│   │   │   ├── dependency.rs            # Dependency エンティティ
│   │   │   ├── health_status.rs         # HealthStatus エンティティ
│   │   │   ├── scorecard.rs             # Scorecard エンティティ
│   │   │   └── claims.rs                # JWT Claims
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── service_repository.rs    # ServiceRepository トレイト
│   │       ├── team_repository.rs       # TeamRepository トレイト
│   │       ├── dependency_repository.rs # DependencyRepository トレイト
│   │       ├── health_repository.rs     # HealthRepository トレイト
│   │       ├── doc_repository.rs        # DocRepository トレイト
│   │       └── scorecard_repository.rs  # ScorecardRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── list_services.rs             # サービス一覧取得
│   │   ├── get_service.rs               # サービス詳細取得
│   │   ├── register_service.rs          # サービス登録
│   │   ├── update_service.rs            # サービス更新
│   │   ├── delete_service.rs            # サービス削除
│   │   ├── manage_dependencies.rs       # 依存関係管理（サイクル検出含む）
│   │   ├── health_status.rs             # ヘルスステータス取得・報告
│   │   ├── manage_docs.rs               # ドキュメント管理
│   │   ├── get_scorecard.rs             # スコアカード取得
│   │   └── search_services.rs           # サービス検索
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── service_handler.rs       # サービス関連 REST ハンドラー
│   │   │   ├── team_handler.rs          # チーム関連 REST ハンドラー
│   │   │   ├── dependency_handler.rs    # 依存関係 REST ハンドラー
│   │   │   ├── health_handler.rs        # ヘルス REST ハンドラー
│   │   │   ├── doc_handler.rs           # ドキュメント REST ハンドラー
│   │   │   ├── scorecard_handler.rs     # スコアカード REST ハンドラー
│   │   │   ├── search_handler.rs        # 検索 REST ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   └── service_catalog_service.rs  # gRPC サービス実装
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── database.rs                  # DB 接続プール管理
│       ├── repository/
│       │   ├── mod.rs
│       │   ├── service_postgres.rs      # ServiceRepository PostgreSQL 実装
│       │   ├── team_postgres.rs         # TeamRepository PostgreSQL 実装
│       │   ├── dependency_postgres.rs   # DependencyRepository PostgreSQL 実装
│       │   ├── health_postgres.rs       # HealthRepository PostgreSQL 実装
│       │   ├── doc_postgres.rs          # DocRepository PostgreSQL 実装
│       │   └── scorecard_postgres.rs    # ScorecardRepository PostgreSQL 実装
│       ├── kafka/                       # 将来実装予定（現在未実装）
│       │   ├── mod.rs
│       │   └── event_publisher.rs       # Kafka イベント発行（将来実装予定）
│       ├── cache/                       # 将来実装予定（現在未実装）
│       │   ├── mod.rs
│       │   └── redis_cache.rs           # Redis キャッシュ（将来実装予定）
│       └── health_collector/
│           ├── mod.rs
│           └── poller.rs                # バックグラウンドヘルスポーリング
├── proto/
│   └── service_catalog.proto            # gRPC プロトコル定義
├── build.rs                             # tonic-build（proto コンパイル）
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── Cargo.toml
├── Cargo.lock
└── Dockerfile
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# gRPC
tonic = "0.12"
prost = "0.13"

# Kafka（将来実装予定・現在未実装）
# rdkafka = { version = "0.37", features = ["cmake-build"] }

# Redis（将来実装予定・現在未実装）
# redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }

[build-dependencies]
tonic-build = "0.12"
```

---

## エンティティ定義

### Service

```rust
// src/domain/entity/service.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub owner_team_id: String,
    pub lifecycle: Lifecycle,
    pub tier: Tier,
    pub repository_url: Option<String>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    Development,
    Staging,
    Production,
    Deprecated,
}

// H-18 監査対応: Tier enum は実装コード（service.rs）の ServiceTier と一致させる。
// System/Business/Service は旧定義。実装では Critical/Standard/Internal を使用している。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Critical,
    Standard,
    Internal,
}
```

### Team

```rust
// src/domain/entity/team.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub slack_channel: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Dependency

```rust
// src/domain/entity/dependency.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub source_service_id: String,
    pub target_service_id: String,
    pub dependency_type: DependencyType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Runtime,
    Build,
    Optional,
}
```

### HealthStatus

```rust
// src/domain/entity/health_status.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub id: String,
    pub service_id: String,
    pub status: HealthState,
    pub last_check_at: DateTime<Utc>,
    pub details: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

### Scorecard

```rust
// src/domain/entity/scorecard.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scorecard {
    pub id: String,
    pub service_id: String,
    pub documentation_score: i32,
    pub test_coverage: i32,
    pub slo_compliance: i32,
    pub security_score: i32,
    pub overall_score: i32,
    pub evaluated_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## ユースケース

### ユースケース一覧

| ユースケース | 説明 | 主要リポジトリ |
| --- | --- | --- |
| `ListServicesUseCase` | ライフサイクル・ティア・タグでサービス一覧を取得 | `ServiceRepository` |
| `GetServiceUseCase` | ID 指定でサービス詳細を取得 | `ServiceRepository` |
| `RegisterServiceUseCase` | 新しいサービスを登録し、イベントを発行 | `ServiceRepository`, `EventPublisher` |
| `UpdateServiceUseCase` | サービス情報を更新し、イベントを発行 | `ServiceRepository`, `EventPublisher` |
| `DeleteServiceUseCase` | サービスを削除し、イベントを発行 | `ServiceRepository`, `EventPublisher` |
| `ManageDependenciesUseCase` | 依存関係の取得・更新・サイクル検出 | `DependencyRepository` |
| `HealthStatusUseCase` | ヘルスステータスの取得・報告 | `HealthRepository` |
| `ManageDocsUseCase` | ドキュメントリンクの取得・更新 | `DocRepository` |
| `GetScorecardUseCase` | スコアカードの取得 | `ScorecardRepository` |
| `SearchServicesUseCase` | タグ・ティア・検索クエリでサービスを横断検索 | `ServiceRepository` |

### ユースケース構造体

```rust
// src/usecase/manage_dependencies.rs
pub struct ManageDependenciesUseCase {
    dependency_repo: Arc<dyn DependencyRepository>,
    service_repo: Arc<dyn ServiceRepository>,
}

impl ManageDependenciesUseCase {
    pub async fn get_dependencies(
        &self,
        service_id: &str,
    ) -> Result<Vec<Dependency>, ServiceCatalogError>;

    pub async fn update_dependencies(
        &self,
        service_id: &str,
        dependencies: Vec<DependencyInput>,
    ) -> Result<Vec<Dependency>, ServiceCatalogError>;
}
```

```rust
// src/usecase/register_service.rs
pub struct RegisterServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl RegisterServiceUseCase {
    pub async fn execute(
        &self,
        input: RegisterServiceInput,
    ) -> Result<Service, ServiceCatalogError>;
}
```

---

## 依存関係サイクル検出

依存関係の更新時に DFS（深さ優先探索）ベースのサイクル検出を行い、循環依存を防止する。

```rust
// src/usecase/manage_dependencies.rs（サイクル検出ロジック）
fn detect_cycle(
    adjacency: &HashMap<String, Vec<String>>,
    start: &str,
) -> bool {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    dfs(adjacency, start, &mut visited, &mut stack)
}

fn dfs(
    adjacency: &HashMap<String, Vec<String>>,
    node: &str,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    stack.insert(node.to_string());

    if let Some(neighbors) = adjacency.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor.as_str()) {
                if dfs(adjacency, neighbor, visited, stack) {
                    return true;
                }
            } else if stack.contains(neighbor.as_str()) {
                return true; // サイクル検出
            }
        }
    }

    stack.remove(node);
    false
}
```

サイクルが検出された場合は `SYS_SCAT_DEPENDENCY_CYCLE` エラーを返す。

---

## ヘルスコレクター

バックグラウンドタスクとして各サービスの `/healthz` エンドポイントをポーリングし、`service_health` テーブルを更新する。

```rust
// src/infrastructure/health_collector/poller.rs
pub struct HealthCollector {
    service_repo: Arc<dyn ServiceRepository>,
    health_repo: Arc<dyn HealthRepository>,
    http_client: reqwest::Client,
    poll_interval: Duration,
}

impl HealthCollector {
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.poll_interval);
        loop {
            interval.tick().await;
            self.poll_all_services().await;
        }
    }

    async fn poll_all_services(&self) {
        let services = self.service_repo
            .list(None, None, None, None)
            .await
            .unwrap_or_default();

        for service in services {
            let status = self.check_health(&service).await;
            let _ = self.health_repo.upsert(&HealthStatus {
                service_id: service.id.clone(),
                status,
                last_check_at: Utc::now(),
                ..Default::default()
            }).await;
        }
    }

    async fn check_health(&self, service: &Service) -> HealthState {
        // サービスの metadata から healthz URL を取得してポーリング
        // タイムアウト: 5秒、リトライなし
        // 200 → Healthy, 503 → Degraded, その他 → Unhealthy, 接続不可 → Unknown
    }
}
```

ポーリング間隔はデフォルト 60 秒。`config.yaml` の `health_collector.poll_interval_secs` で設定可能。

---

## イベント発行

> **注意（M-20 監査対応）:** Kafka EventPublisher は将来実装予定です。現在は未実装であり、以下のコードはインターフェース定義のみです。サービスの登録・更新・削除時のイベント発行は実装完了後に有効になります。

サービスの登録・更新・削除時に Kafka イベントを発行する（将来実装予定）。

### イベントトピック

| トピック | トリガー | ペイロード |
| --- | --- | --- |
| `service.registered` | サービス登録時 | `{ service_id, name, owner_team_id, lifecycle, tier }` |
| `service.updated` | サービス更新時 | `{ service_id, name, changed_fields }` |
| `service.deleted` | サービス削除時 | `{ service_id, name }` |

```rust
// src/infrastructure/kafka/event_publisher.rs
pub struct KafkaEventPublisher {
    producer: FutureProducer,
}

impl KafkaEventPublisher {
    pub async fn publish(
        &self,
        topic: &str,
        key: &str,
        payload: &serde_json::Value,
    ) -> Result<(), ServiceCatalogError>;
}
```

---

## リポジトリ実装

### リポジトリトレイト

```rust
// src/domain/repository/service_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn list(
        &self,
        lifecycle: Option<&str>,
        tier: Option<&str>,
        tag: Option<&str>,
        owner_team_id: Option<&str>,
    ) -> anyhow::Result<Vec<Service>>;
    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<Service>>;
    async fn create(&self, service: &Service) -> anyhow::Result<()>;
    async fn update(&self, service: &Service) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn search(&self, query: &str, tag: Option<&str>, tier: Option<&str>, lifecycle: Option<&str>) -> anyhow::Result<Vec<Service>>;
}

// src/domain/repository/dependency_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DependencyRepository: Send + Sync {
    async fn list_by_service_id(&self, service_id: &str) -> anyhow::Result<Vec<Dependency>>;
    async fn replace_for_service(&self, service_id: &str, deps: &[Dependency]) -> anyhow::Result<()>;
    async fn list_all(&self) -> anyhow::Result<Vec<Dependency>>;
}

// src/domain/repository/health_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HealthRepository: Send + Sync {
    async fn get_by_service_id(&self, service_id: &str) -> anyhow::Result<Option<HealthStatus>>;
    async fn upsert(&self, status: &HealthStatus) -> anyhow::Result<()>;
}
```

---

## config.yaml

サービスカタログサーバー固有の設定セクション。共通セクション（app/server/database/observability）は [Rust共通実装.md](../../_common/Rust共通実装.md#共通configyaml) を参照。

```yaml
# cache（将来実装予定・現在未実装）:
#   redis_url: "redis://redis.system.svc.cluster.local:6379/0"
#   ttl_secs: 300

# kafka（将来実装予定・現在未実装）:
#   brokers: "kafka.system.svc.cluster.local:9092"
#   topic_prefix: "service-catalog"

health_collector:
  poll_interval_secs: 60
  timeout_secs: 5

grpc:
  port: 50051
```

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [database.md](database.md) -- データベース設計
- [deploy.md](deploy.md) -- DB マイグレーション・テスト・Dockerfile・Helm values
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
