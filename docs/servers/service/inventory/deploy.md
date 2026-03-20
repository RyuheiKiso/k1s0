# service-inventory-server デプロイ設計

service-inventory-server の Dockerfile・テスト・CI/CD パイプライン・設定ファイル・Helm values を定義する。概要・API 定義・アーキテクチャは [service-inventory-server.md](server.md) を参照。

> **適用範囲**: 本ドキュメントは **service Tier の inventory サーバー** に適用されるデプロイ設計である。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | k1s0-inventory-server |
| Tier | service |
| 実装言語 | Rust |
| REST ポート | 8311 |
| gRPC ポート | 50072 |
| DB | PostgreSQL（`inventory_service` スキーマ） |
| メッセージング | Kafka（在庫予約・解放イベント） |
| ビルドコンテキスト | リポジトリルート（system ライブラリ依存解決のため） |

---

## コンテナイメージ

### Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストはリポジトリルート（system ライブラリ依存解決のため）。

```dockerfile
# Build stage
# Note: build context is repo root (to include system library dependencies)
FROM rust:1.93-bookworm AS builder

# cmake + build-essential（rdkafka cmake-build feature 用）のインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# system ライブラリ依存を含めるためリポジトリの regions ディレクトリをコピー
COPY regions regions

# ローカル開発用ビルド引数: 認証バイパスを有効化するには
# CARGO_FEATURES="k1s0-server-common/dev-auth-bypass" を指定する
ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release --manifest-path regions/service/inventory/server/rust/inventory/Cargo.toml; \
    fi

FROM busybox:1.36.1-musl AS busybox

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/regions/service/inventory/server/rust/inventory/target/release/k1s0-inventory-server /k1s0-inventory-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8311

ENTRYPOINT ["/k1s0-inventory-server"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| busybox | ヘルスチェック用のシェルコマンド提供 |
| ビルドコマンド | `cargo build --release --manifest-path` で inventory パッケージを指定 |
| ビルドコンテキスト | リポジトリルート（`COPY regions regions` で system ライブラリ依存を含める） |
| 公開ポート | 8311（REST API） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

---

## 環境変数

inventory-server は config.yaml ベースで設定を管理する。環境変数による上書きも可能。

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_ENVIRONMENT` | 実行環境（development / docker / production） | `development` |
| `DATABASE_URL` | PostgreSQL 接続 URL（Docker Compose 用） | - |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス | `localhost:9092` |
| `AUTH_JWKS_URL` | JWKS エンドポイント URL | - |

### config.yaml（開発環境）

`regions/service/inventory/server/rust/inventory/config/default.yaml` に配置。

```yaml
app:
  name: "k1s0-inventory-server"
  version: "0.1.0"
  environment: "development"

server:
  host: "0.0.0.0"
  port: 8311
  grpc_port: 50072

database:
  host: "localhost"
  port: 5432
  name: "k1s0_inventory"
  schema: "inventory_service"
  user: "k1s0"
  password: "k1s0"
  ssl_mode: "disable"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300

kafka:
  brokers:
    - "localhost:9092"
  inventory_reserved_topic: "k1s0.service.inventory.reserved.v1"
  inventory_released_topic: "k1s0.service.inventory.released.v1"
  security_protocol: "PLAINTEXT"

auth:
  jwks_url: "http://localhost:8080/realms/k1s0/protocol/openid-connect/certs"
  issuer: "http://localhost:8080/realms/k1s0"
  audience: "k1s0-service"
  jwks_cache_ttl_secs: 300

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "http://otel-collector.observability:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
```

### config.yaml（本番環境）

```yaml
app:
  name: "k1s0-inventory-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8311
  grpc_port: 50072

database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_inventory"
  schema: "inventory_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300

kafka:
  brokers:
    - "kafka.k1s0-infra.svc.cluster.local:9092"
  inventory_reserved_topic: "k1s0.service.inventory.reserved.v1"
  inventory_released_topic: "k1s0.service.inventory.released.v1"
  security_protocol: "PLAINTEXT"

auth:
  jwks_url: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
  issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
  audience: "k1s0-service"
  jwks_cache_ttl_secs: 300

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "http://otel-collector.observability:4317"
    sample_rate: 0.1
  metrics:
    enabled: true
    path: "/metrics"
```

---

## DB マイグレーション

inventory-server は PostgreSQL の `inventory_service` スキーマを使用する。マイグレーションファイルは `regions/service/inventory/database/postgres/migrations/` に配置。

### マイグレーション一覧

| ファイル | 説明 |
| --- | --- |
| `001_create_inventory_items.up.sql` | 在庫アイテムテーブル作成（product_id + warehouse_id の一意制約付き） |
| `002_create_outbox.up.sql` | Outbox パターン用テーブル作成 |
| `003_add_outbox_idempotency_key.up.sql` | Outbox テーブルに冪等キーカラムを追加 |

### 主要スキーマ

```sql
-- inventory_service スキーマ
CREATE SCHEMA IF NOT EXISTS inventory_service;

-- 在庫アイテムテーブル
CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY,
    product_id TEXT NOT NULL,
    warehouse_id TEXT NOT NULL,
    qty_available INT NOT NULL DEFAULT 0,
    qty_reserved INT NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_product_warehouse UNIQUE (product_id, warehouse_id),
    CONSTRAINT chk_qty_available CHECK (qty_available >= 0),
    CONSTRAINT chk_qty_reserved CHECK (qty_reserved >= 0)
);

CREATE INDEX idx_inventory_product ON inventory_items (product_id);
CREATE INDEX idx_inventory_warehouse ON inventory_items (warehouse_id);
```

詳細は [service-inventory-database.md](database.md) を参照。

---

## ヘルスチェック

| エンドポイント | 用途 | 認証 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe（プロセス生存確認） | 不要 |
| `GET /readyz` | Readiness Probe（DB・Kafka 接続確認） | 不要 |
| `GET /metrics` | Prometheus メトリクス取得 | 不要 |

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/database | 統合テスト（DB） | `testcontainers` |
| infrastructure/kafka | 統合テスト | `mockall` |

---

## CI/CD パイプライン

### CI（`.github/workflows/inventory-ci.yaml`）

PR 時に `regions/service/inventory/server/rust/inventory/**` の変更を検出してトリガーする。

```
detect-changes → lint-rust → test-rust → build-rust → security-scan
```

| ジョブ | 内容 |
| --- | --- |
| `detect-changes` | `dorny/paths-filter@v3` でパス変更検出 |
| `lint-rust` | `cargo fmt --check` + `cargo clippy -D warnings` |
| `test-rust` | `cargo test --all` |
| `build-rust` | `cargo build --release` |
| `security-scan` | Trivy ファイルシステムスキャン（HIGH/CRITICAL） |

### CD（`.github/workflows/inventory-deploy.yaml`）

main ブランチへの push 時にトリガー。

```
build-and-push → deploy-dev → deploy-staging → deploy-prod（手動承認）
```

| ジョブ | 内容 |
| --- | --- |
| `build-and-push` | Docker Buildx ビルド、Harbor プッシュ、Cosign 署名、Trivy イメージスキャン、SBOM 生成 |
| `deploy-dev` | Cosign 署名検証 → Helm upgrade（dev 環境） |
| `deploy-staging` | Cosign 署名検証 → Helm upgrade（staging 環境） |
| `deploy-prod` | Cosign 署名検証 → Helm upgrade（prod 環境、手動承認必須） |

### イメージタグ戦略

| タグ | 形式 |
| --- | --- |
| バージョン | `{VERSION}`（git describe から取得） |
| バージョン + SHA | `{VERSION}-{SHORT_SHA}` |
| latest | `latest` |

**レジストリ**: `harbor.internal.example.com/k1s0-service/inventory`

---

## ポートマッピング

| 環境 | ホストポート | コンテナポート | プロトコル |
| --- | --- | --- | --- |
| docker-compose | 8311 | 8311 | REST API |
| docker-compose | 50072 | 50072 | gRPC |
| Kubernetes | 80（Service） | 8311（Pod） | REST API |
| Kubernetes | 50072（Service） | 50072（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | 在庫アイテム・Outbox テーブルの永続化 | Yes |
| Kafka | 在庫イベント配信（`k1s0.service.inventory.reserved.v1` / `k1s0.service.inventory.released.v1`） | Yes |
| Keycloak | JWT 認証（JWKS エンドポイント） | Yes |

---

## リソース要件

### Helm Chart

Helm Chart は `infra/helm/services/service/inventory/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Chart 構成

```
infra/helm/services/service/inventory/
  Chart.yaml
  Chart.lock
  values.yaml           # デフォルト values
  values-dev.yaml       # dev 環境オーバーライド
  values-staging.yaml   # staging 環境オーバーライド
  values-prod.yaml      # prod 環境オーバーライド
  charts/
    k1s0-common-0.1.0.tgz
  templates/
    _helpers.tpl
    configmap.yaml
    deployment.yaml
    hpa.yaml
    pdb.yaml
    service.yaml
    serviceaccount.yaml
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/service/inventory/database` |
| Kafka SASL | `secret/data/k1s0/service/kafka/sasl` |

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-service/inventory
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8311
  grpcPort: 50072

service:
  type: ClusterIP
  port: 80
  grpcPort: 50072

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 8
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

pdb:
  enabled: true
  minAvailable: 1

vault:
  enabled: true
  role: "service"
  secrets:
    - path: "secret/data/k1s0/service/inventory/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/service/kafka/sasl"
      key: "password"
      mountPath: "/vault/secrets/kafka-sasl"

labels:
  tier: service
```

### dev 環境オーバーライド

```yaml
replicaCount: 1

resources:
  requests:
    cpu: 50m
    memory: 64Mi
  limits:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: false

pdb:
  enabled: false

vault:
  enabled: false
```

---

## スケーリング設定

| 項目 | 値 |
| --- | --- |
| 最小レプリカ数 | 2 |
| 最大レプリカ数 | 8 |
| CPU スケーリング閾値 | 70% |
| メモリスケーリング閾値 | 80% |
| PDB minAvailable | 1 |

在庫予約・解放はトランザクション頻度が高いため、最大レプリカ数を 8 に設定。Outbox Pattern による at-least-once delivery を採用しているため、スケールアウト時もイベント配信の整合性を維持できる。

---

## セキュリティ設定

```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 65532
  fsGroup: 65532

containerSecurityContext:
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop: ["ALL"]
```

---

## Kong ルーティング

```yaml
services:
  - name: inventory-v1
    url: http://inventory.k1s0-service.svc.cluster.local:80
    routes:
      - name: inventory-v1-route
        paths:
          - /api/v1/inventory
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## Kubernetes Probes

```yaml
# Liveness Probe
livenessProbe:
  httpGet:
    path: /healthz
    port: 8311
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe
readinessProbe:
  httpGet:
    path: /readyz
    port: 8311
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

---

## Docker Compose（開発環境）

```yaml
inventory-server:
  build:
    context: .
    dockerfile: regions/service/inventory/server/rust/inventory/Dockerfile
  ports:
    - "8311:8311"
    - "50072:50072"
  environment:
    - DATABASE_URL=postgresql://k1s0:k1s0@postgres:5432/k1s0_inventory
    - KAFKA_BROKERS=kafka:9092
    - AUTH_JWKS_URL=http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs
  depends_on:
    - postgres
    - kafka
    - keycloak
```

---

## 関連ドキュメント

- [service-inventory-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [inventory データベース設計](database.md) -- データベーススキーマ・マイグレーション
- [inventory 実装設計](implementation.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
