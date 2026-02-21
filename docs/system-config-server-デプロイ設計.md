# system-config-server デプロイ設計

system-config-server のキャッシュ戦略・DB マイグレーション・テスト・Dockerfile・Helm values を定義する。概要・API 定義・アーキテクチャは [system-config-server設計.md](system-config-server設計.md) を参照。

---

## キャッシュ戦略

設定値の取得は高頻度で呼び出されるため、インメモリキャッシュによるレイテンシ削減を行う。

### キャッシュ方針

| 項目 | 値 |
| --- | --- |
| キャッシュ方式 | インメモリ（Go: ristretto, Rust: moka） |
| TTL | 設定可能（デフォルト 60 秒） |
| 最大エントリ数 | 設定可能（デフォルト 10,000） |
| キャッシュキー | `{namespace}:{key}` 形式 |
| 無効化タイミング | PUT / DELETE 実行時に即座に無効化 |
| キャッシュミス | DB から取得後にキャッシュに格納 |

### キャッシュ無効化フロー

```
1. PUT /api/v1/config/:namespace/:key が呼ばれる
2. DB を更新（楽観的排他制御によるバージョン検証）
3. config_change_logs テーブルに変更ログを記録
4. インメモリキャッシュの該当エントリを無効化
5. Kafka トピック k1s0.system.config.changed.v1 にイベントを発行
6. 他サービスは Kafka イベントを受信してローカルキャッシュを無効化
```

### Rust キャッシュ実装例

```rust
// src/infra/cache/config_cache.rs
use moka::future::Cache;
use std::time::Duration;

use crate::domain::model::ConfigEntry;

pub struct ConfigCache {
    cache: Cache<String, ConfigEntry>,
}

impl ConfigCache {
    pub fn new(ttl_seconds: u64, max_entries: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<ConfigEntry> {
        self.cache.get(key).await
    }

    pub async fn set(&self, key: &str, entry: &ConfigEntry) {
        self.cache.insert(key.to_string(), entry.clone()).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }
}
```

---

## データベースマイグレーション

設定値と変更ログの2テーブルを PostgreSQL（config-db）に格納する。

```sql
-- migrations/001_create_config_entries.sql

CREATE TABLE IF NOT EXISTS config_entries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace   VARCHAR(255) NOT NULL,
    key         VARCHAR(255) NOT NULL,
    value       JSONB        NOT NULL,
    version     INTEGER      NOT NULL DEFAULT 1,
    description TEXT         NOT NULL DEFAULT '',
    created_by  VARCHAR(255) NOT NULL,
    updated_by  VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_config_entries_namespace_key UNIQUE (namespace, key)
);

-- 検索用インデックス
CREATE INDEX idx_config_entries_namespace ON config_entries (namespace);
CREATE INDEX idx_config_entries_key ON config_entries (key);
CREATE INDEX idx_config_entries_updated_at ON config_entries (updated_at DESC);

-- サービス名検索用（namespace の第2階層がサービス名に対応）
-- 例: system.auth.database → auth がサービス名
CREATE INDEX idx_config_entries_namespace_prefix ON config_entries USING btree (namespace varchar_pattern_ops);

COMMENT ON TABLE config_entries IS '設定値エントリ。namespace.key の一意制約で管理。';
COMMENT ON COLUMN config_entries.namespace IS 'Tier.Service.Section 形式の名前空間（例: system.auth.database）';
COMMENT ON COLUMN config_entries.value IS 'JSONB 形式の設定値。string, number, boolean, object を格納可能';
COMMENT ON COLUMN config_entries.version IS '楽観的排他制御用のバージョン番号。更新のたびにインクリメント';
```

```sql
-- migrations/002_create_config_change_logs.sql

CREATE TABLE IF NOT EXISTS config_change_logs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_entry_id  UUID         NOT NULL,
    namespace        VARCHAR(255) NOT NULL,
    key              VARCHAR(255) NOT NULL,
    old_value        JSONB,
    new_value        JSONB,
    old_version      INTEGER      NOT NULL DEFAULT 0,
    new_version      INTEGER      NOT NULL,
    change_type      VARCHAR(20)  NOT NULL CHECK (change_type IN ('CREATED', 'UPDATED', 'DELETED')),
    changed_by       VARCHAR(255) NOT NULL,
    changed_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 検索用インデックス
CREATE INDEX idx_config_change_logs_config_entry_id ON config_change_logs (config_entry_id);
CREATE INDEX idx_config_change_logs_namespace ON config_change_logs (namespace);
CREATE INDEX idx_config_change_logs_changed_at ON config_change_logs (changed_at DESC);
CREATE INDEX idx_config_change_logs_changed_by ON config_change_logs (changed_by);
CREATE INDEX idx_config_change_logs_change_type ON config_change_logs (change_type);

-- 複合インデックス（設定エントリ + 日時範囲の検索最適化）
CREATE INDEX idx_config_change_logs_entry_changed ON config_change_logs (config_entry_id, changed_at DESC);

-- パーティショニング（月単位）は運用フェーズで検討
COMMENT ON TABLE config_change_logs IS '設定変更の監査ログ。全ての CRUD 操作を記録。保持期間は 1 年間（可観測性設計.md 参照）';
```

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infra/persistence | 統合テスト（DB） | `testcontainers` |
| infra/cache | 単体テスト | `tokio::test` |

### Rust テスト例

```rust
// src/usecase/get_config.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::MockConfigRepository;
    use crate::infra::cache::ConfigCache;

    #[tokio::test]
    async fn test_get_config_cache_hit() {
        let mut mock_repo = MockConfigRepository::new();
        // キャッシュヒット時は DB 呼び出しなし
        mock_repo.expect_find_by_namespace_and_key().times(0);

        let cache = Arc::new(ConfigCache::new(60, 1000));
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 3,
            description: String::new(),
            created_by: "admin".to_string(),
            updated_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        cache.set("system.auth.database:max_connections", &entry).await;

        let uc = GetConfigUseCase::new(Arc::new(mock_repo), cache);

        let result = uc.execute("system.auth.database", "max_connections").await.unwrap();
        assert_eq!(result.key, "max_connections");
        assert_eq!(result.version, 3);
    }

    #[tokio::test]
    async fn test_get_config_cache_miss() {
        let mut mock_repo = MockConfigRepository::new();
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 3,
            description: String::new(),
            created_by: "admin".to_string(),
            updated_by: "admin".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let entry_clone = entry.clone();

        mock_repo
            .expect_find_by_namespace_and_key()
            .with(eq("system.auth.database"), eq("max_connections"))
            .returning(move |_, _| Ok(Some(entry_clone.clone())));

        let cache = Arc::new(ConfigCache::new(60, 1000));
        let uc = GetConfigUseCase::new(Arc::new(mock_repo), cache.clone());

        let result = uc.execute("system.auth.database", "max_connections").await.unwrap();
        assert_eq!(result.key, "max_connections");

        // キャッシュに格納されていることを確認
        let cached = cache.get("system.auth.database:max_connections").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().version, 3);
    }
}
```

### testcontainers による DB 統合テスト

#### Rust

```rust
// src/infra/persistence/config_repository_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_config_repository_crud() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "config_db_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/config_db_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = ConfigRepositoryImpl::new(pool);

        // Create
        let entry = ConfigEntry {
            id: uuid::Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value: serde_json::json!(25),
            version: 1,
            description: "DB 最大接続数".to_string(),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        repo.create(&entry).await.unwrap();

        // Read
        let found = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.key, "max_connections");
        assert_eq!(found.version, 1);

        // Update
        let mut updated = found.clone();
        updated.value = serde_json::json!(50);
        updated.version = 2;
        repo.update(&updated).await.unwrap();

        // Delete
        repo.delete("system.auth.database", "max_connections")
            .await
            .unwrap();

        let deleted = repo
            .find_by_namespace_and_key("system.auth.database", "max_connections")
            .await
            .unwrap();
        assert!(deleted.is_none());
    }
}
```

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](Dockerイメージ戦略.md) のテンプレートに従う。

#### Rust

```dockerfile
# ---- Build ----
FROM rust:1.82-bookworm AS build
WORKDIR /src

# protoc のインストール（tonic-build に必要）
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/config-server /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。設定管理サーバー固有の values は以下の通り。

```yaml
# values-config.yaml
app:
  name: config-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/config-server
  tag: "0.1.0"

service:
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: grpc
      port: 50051
      targetPort: 50051

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/config/database"
  vault.hashicorp.com/agent-inject-secret-kafka-sasl: "secret/data/k1s0/system/kafka/sasl"

# ヘルスチェック
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5

# ConfigMap マウント
configMap:
  name: config-server-config
  mountPath: /etc/app/config.yaml
```

### Kong ルーティング

[認証認可設計.md](認証認可設計.md) の Kong ルーティング設計に従い、設定管理サーバーを Kong に登録する。

```yaml
services:
  - name: config-v1
    url: http://config-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: config-v1-route
        paths:
          - /api/v1/config
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [system-config-server設計.md](system-config-server設計.md) -- 概要・API 定義・アーキテクチャ
- [system-config-server-実装設計.md](system-config-server-実装設計.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](認証認可設計.md) -- Kong ルーティング設計
