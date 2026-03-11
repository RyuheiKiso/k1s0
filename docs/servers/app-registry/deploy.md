# system-app-registry-server デプロイ設計

system-app-registry-server（アプリケーションレジストリサーバー）の DB マイグレーション・テスト・Dockerfile・Helm values を定義する。概要・API 定義は [server.md](server.md) を参照。

---

## データベースマイグレーション

アプリ・バージョン・ダウンロード統計テーブルは PostgreSQL に格納する。詳細スキーマは [database.md](database.md) 参照。

### マイグレーション一覧

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `app_registry` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `002_create_apps.up.sql` | apps テーブル・インデックス作成 |
| `003_create_app_versions.up.sql` | app_versions テーブル・ユニーク制約・インデックス作成 |
| `004_create_download_stats.up.sql` | download_stats テーブル・インデックス作成 |
| `005_seed_initial_data.up.sql` | 初期アプリデータ投入 |

```sql
-- migrations/001_create_schema.up.sql
CREATE SCHEMA IF NOT EXISTS app_registry;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION app_registry.update_updated_at()
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
| adapter/handler | 統合テスト（HTTP） | `axum::test` + `tokio::test` |
| infrastructure/repository | 統合テスト（DB） | `testcontainers` |
| infrastructure/s3_client | 統合テスト | `mockall` + モック S3 |

### ユースケース単体テスト（mockall）

`GenerateDownloadUrlUseCase` の成功・失敗パターンをモックで検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_download_url_success() {
        let mut mock_version_repo = MockVersionRepository::new();
        mock_version_repo
            .expect_get_by_version()
            .returning(|_, _, _, _| {
                Ok(Some(AppVersion {
                    s3_key: "apps/cli/1.0.0/windows/amd64/cli.exe".to_string(),
                    ..Default::default()
                }))
            });

        let mut mock_stats_repo = MockDownloadStatsRepository::new();
        mock_stats_repo
            .expect_record()
            .returning(|_| Ok(()));

        // S3Client のモックでURL生成を検証
        // ...
    }

    #[tokio::test]
    async fn test_generate_download_url_version_not_found() {
        let mut mock_version_repo = MockVersionRepository::new();
        mock_version_repo
            .expect_get_by_version()
            .returning(|_, _, _, _| Ok(None));

        // SYS_APPS_VERSION_NOT_FOUND エラーを検証
        // ...
    }
}
```

### testcontainers による DB 統合テスト

`AppPostgresRepository` / `VersionPostgresRepository` の CRUD を実 PostgreSQL で検証する。

```rust
#[cfg(test)]
mod tests {
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_app_repository_list_and_get() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "app_registry_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/app_registry_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = AppPostgresRepository::new(pool);
        let apps = repo.list(None, None).await.unwrap();
        // 初期データの検証
        // ...
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

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

RUN cargo build --release -p k1s0-app-registry-server

# Runtime stage
FROM debian:bookworm-slim

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-app-registry-server /k1s0-app-registry-server

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-app-registry-server"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド） |
| ランタイムステージ | `debian:bookworm-slim`（最小イメージ） |
| 追加パッケージ | なし（gRPC なしのため `protobuf-compiler` 不要） |
| ビルドコマンド | `cargo build --release -p k1s0-app-registry-server` |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API のみ） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。アプリレジストリサーバー固有の values は以下の通り。

```yaml
# values-app-registry.yaml
app:
  name: app-registry-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/app-registry-server
  tag: "0.1.0"

service:
  ports:
    - name: http
      port: 80
      targetPort: 8080

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
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/app-registry/database"
  vault.hashicorp.com/agent-inject-secret-s3: "secret/data/k1s0/system/app-registry/s3"

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
  name: app-registry-server-config
  mountPath: /etc/app/config.yaml
```

---

## 共通関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベース設計
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
