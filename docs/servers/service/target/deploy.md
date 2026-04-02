# service-target-server デプロイ設計

service-target-server の Dockerfile・Helm values・デプロイ設定を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | `k1s0-target-server` |
| Tier | service |
| 実装言語 | Rust |
| REST ポート | 8080（コンテナ内） |
| gRPC ポート | 50051（コンテナ内） |
| DB | PostgreSQL（`target_service` スキーマ） |
| Kafka | プロデューサー（`k1s0.service.target.progress_updated.v1`） |
| イメージレジストリ | `harbor.internal.example.com/k1s0-service/target` |
| ビルドコンテキスト | `regions/service` |

board-server は Kanban ボードのカラム管理・WIP制限を REST API で提供し、カラム更新イベントを Outbox pattern で Kafka に非同期配信するサービスである。楽観的ロックによる同時更新制御を実装する。

> **注記**: board-server は service tier に属するため、ビルドコンテキストは `regions/service` となる。system ライブラリへの参照は相対パスで解決される。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm` + `cargo-chef`（依存キャッシュ 4 ステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| ビルドコマンド | `cargo build --release -p k1s0-target-server` |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot` |

### Dockerfile

```dockerfile
# chef ベースステージ: cargo-chef とビルド依存のインストール
FROM rust:1.93-bookworm AS chef

RUN cargo install cargo-chef --locked

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-target-server

FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-target-server

FROM busybox:1.36.1-musl AS busybox

FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-target-server /k1s0-target-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-target-server"]
```

---

## 環境変数

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_NAME` | アプリケーション名 | `k1s0-target-server` |
| `APP_ENVIRONMENT` | 実行環境（`dev` / `staging` / `production`） | `dev` |
| `SERVER_HOST` | バインドアドレス | `0.0.0.0` |
| `SERVER_PORT` | REST API ポート | `8080` |
| `DATABASE_URL` | PostgreSQL 接続文字列 | - |
| `DATABASE_SCHEMA` | スキーマ名 | `target_service` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス（カンマ区切り） | - |
| `AUTH_JWKS_URL` | JWT 検証用 JWKS URL | - |
| `AUTH_ISSUER` | JWT issuer | - |
| `AUTH_AUDIENCE` | JWT audience | - |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry Collector エンドポイント | - |
| `RUST_LOG` | ログレベル | `info` |

### Vault シークレット

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/service/target/database` |
| Kafka SASL | `secret/data/k1s0/service/kafka/sasl` |

---

## DB マイグレーション

ボードデータは PostgreSQL の `target_service` スキーマに格納する。詳細スキーマは [database.md](database.md) 参照。

マイグレーションは `sqlx migrate run` で実行する。Advisory Lock ID: `1000000014`。

Docker Compose 起動時のスキーマ初期化: `infra/docker/init-db/16-target-schema.sql`

---

## ヘルスチェック

| エンドポイント | パス | ポート | 用途 |
| --- | --- | --- | --- |
| Liveness | `/healthz` | 8080 | プロセス生存確認 |
| Readiness | `/readyz` | 8080 | DB 接続確認を含むトラフィック受付可否 |
| Metrics | `/metrics` | 8080 | Prometheus メトリクス |

### Kubernetes Probes

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

---

## リソース要件

### 本番環境

| リソース | requests | limits |
| --- | --- | --- |
| CPU | 100m | 500m |
| Memory | 128Mi | 512Mi |

### dev 環境

| リソース | requests | limits |
| --- | --- | --- |
| CPU | 50m | 250m |
| Memory | 64Mi | 256Mi |

---

## スケーリング設定

### HPA（Horizontal Pod Autoscaler）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| 最小レプリカ数 | 2 | 1 |
| 最大レプリカ数 | 10 | 1 |
| CPU 閾値 | 70% | - |

### PDB（PodDisruptionBudget）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| minAvailable | 1 | - |

---

## Helm Chart

Helm Chart は `infra/helm/services/service/board/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-service/target
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8080

service:
  type: ClusterIP
  port: 80

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
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

pdb:
  enabled: true
  minAvailable: 1

vault:
  enabled: true
  role: "service"
  secrets:
    - path: "secret/data/k1s0/service/target/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"

kafka:
  enabled: true
  brokers: []

labels:
  tier: service
```

### dev 環境オーバーライド（Docker Compose）

```yaml
board-rust:
  build:
    context: .
    dockerfile: ./regions/service/target/server/rust/board/Dockerfile
  profiles: [service]
  ports:
    - "${TARGET_REST_HOST_PORT:-8340}:8080"
    - "${TARGET_GRPC_HOST_PORT:-9340}:50051"
  environment:
    - CONFIG_PATH=/app/config/default.yaml
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
  volumes:
    - ./regions/service/target/server/rust/board/config:/app/config
```

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

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | ボードカラム情報の永続化 | Yes |
| Kafka | ボードカラム更新イベント配信 | No（未設定時は NoopTargetEventPublisher） |
| Keycloak | JWT 認証 | Yes |

---

## CI/CD パイプライン

### CI（`.github/workflows/target-ci.yaml`）

変更トリガーパス:
- `regions/service/target/server/**`
- `regions/service/target/client/**`
- `regions/service/target/database/**`

```
detect-changes → lint-rust → test-rust → build-rust → security-scan
```

### CD（`.github/workflows/target-deploy.yaml`）

```
build-and-push → migration → deploy-dev → deploy-staging → deploy-prod（手動承認）
```

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
