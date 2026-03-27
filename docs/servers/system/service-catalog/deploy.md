# system-service-catalog-server デプロイ設計

system-service-catalog-server（サービスカタログサーバー）の DB マイグレーション・テスト・Dockerfile・Helm values を定義する。概要・API 定義は [server.md](server.md) を参照。

---

## データベースマイグレーション

サービス・チーム・依存関係・ヘルス・ドキュメント・スコアカードテーブルは PostgreSQL に格納する。詳細スキーマは [database.md](database.md) 参照。

### マイグレーション一覧

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `service_catalog` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `002_create_teams.up.sql` | teams テーブル・インデックス作成 |
| `003_create_services.up.sql` | services テーブル・制約・GIN インデックス作成 |
| `004_create_service_dependencies.up.sql` | service_dependencies テーブル・ユニーク制約・インデックス作成 |
| `005_create_service_health.up.sql` | service_health テーブル・インデックス作成 |
| `006_create_service_docs.up.sql` | service_docs テーブル・インデックス作成 |
| `007_create_service_scorecards.up.sql` | service_scorecards テーブル・CHECK 制約・インデックス作成 |

```sql
-- migrations/001_create_schema.up.sql
CREATE SCHEMA IF NOT EXISTS service_catalog;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION service_catalog.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | Rust |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| usecase/manage_dependencies | 単体テスト（サイクル検出） | `#[cfg(test)]` + グラフテストケース |
| adapter/handler | 統合テスト（HTTP） | `axum::test` + `tokio::test` |
| adapter/grpc | 統合テスト（gRPC） | `tonic::transport::Channel` |
| infrastructure/repository | 統合テスト（DB） | `testcontainers` |
| infrastructure/kafka | 統合テスト | `mockall` + モック Producer |
| infrastructure/cache | 統合テスト | `testcontainers`（Redis） |
| infrastructure/health_collector | 統合テスト | `mockall` + `wiremock` |

### ユースケース単体テスト（mockall）

`ManageDependenciesUseCase` のサイクル検出パターンをモックで検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_dependencies_success() {
        let mut mock_dep_repo = MockDependencyRepository::new();
        mock_dep_repo
            .expect_list_all()
            .returning(|| Ok(vec![]));
        mock_dep_repo
            .expect_replace_for_service()
            .returning(|_, _| Ok(()));

        let mut mock_service_repo = MockServiceRepository::new();
        mock_service_repo
            .expect_get_by_id()
            .returning(|_| Ok(Some(Service::default())));

        let usecase = ManageDependenciesUseCase::new(
            Arc::new(mock_dep_repo),
            Arc::new(mock_service_repo),
        );

        let result = usecase.update_dependencies("svc-a", vec![
            DependencyInput {
                target_service_id: "svc-b".to_string(),
                dependency_type: DependencyType::Runtime,
                description: None,
            },
        ]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_dependencies_cycle_detected() {
        let mut mock_dep_repo = MockDependencyRepository::new();
        mock_dep_repo
            .expect_list_all()
            .returning(|| Ok(vec![
                Dependency {
                    source_service_id: "svc-b".to_string(),
                    target_service_id: "svc-a".to_string(),
                    ..Default::default()
                },
            ]));

        let mut mock_service_repo = MockServiceRepository::new();
        mock_service_repo
            .expect_get_by_id()
            .returning(|_| Ok(Some(Service::default())));

        let usecase = ManageDependenciesUseCase::new(
            Arc::new(mock_dep_repo),
            Arc::new(mock_service_repo),
        );

        // svc-a → svc-b → svc-a のサイクル
        let result = usecase.update_dependencies("svc-a", vec![
            DependencyInput {
                target_service_id: "svc-b".to_string(),
                dependency_type: DependencyType::Runtime,
                description: None,
            },
        ]).await;

        assert!(matches!(result, Err(ServiceCatalogError::DependencyCycle)));
    }
}
```

### testcontainers による DB 統合テスト

`ServicePostgresRepository` の CRUD を実 PostgreSQL で検証する。

```rust
#[cfg(test)]
mod tests {
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_service_repository_crud() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "service_catalog_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/service_catalog_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = ServicePostgresRepository::new(pool);

        // Create
        let service = Service { name: "test-svc".to_string(), ..Default::default() };
        repo.create(&service).await.unwrap();

        // Read
        let found = repo.get_by_id(&service.id).await.unwrap();
        assert!(found.is_some());

        // List
        let services = repo.list(None, None, None, None).await.unwrap();
        assert!(!services.is_empty());

        // Delete
        let deleted = repo.delete(&service.id).await.unwrap();
        assert!(deleted);
    }
}
```

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストは `regions/system`（ライブラリ依存解決のため）。

```dockerfile
# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.93-bookworm AS builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

RUN cargo build --release -p k1s0-service-catalog-server

# Runtime stage
FROM debian:bookworm-slim

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-service-catalog-server /k1s0-service-catalog-server

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-service-catalog-server"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド） |
| ランタイムステージ | `debian:bookworm-slim`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（gRPC proto コンパイル用） |
| ビルドコマンド | `cargo build --release -p k1s0-service-catalog-server` |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。サービスカタログサーバー固有の values は以下の通り。

```yaml
# values-service-catalog.yaml
app:
  name: service-catalog-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/service-catalog-server
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
    cpu: 200m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 512Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/service-catalog/database"
  vault.hashicorp.com/agent-inject-secret-redis: "secret/data/k1s0/system/service-catalog/redis"
  vault.hashicorp.com/agent-inject-secret-kafka: "secret/data/k1s0/system/service-catalog/kafka"

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
  name: service-catalog-server-config
  mountPath: /etc/app/config.yaml
```

---

## 共通関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../../_common/deploy.md#共通関連ドキュメント) を参照。

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベース設計
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
