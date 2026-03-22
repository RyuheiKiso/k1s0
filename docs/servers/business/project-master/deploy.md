# business-project-master-server デプロイ仕様

business-project-master-server の Dockerfile・Helm values・デプロイ設定を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | `k1s0-project-master-server` |
| Tier | business |
| 実装言語 | Rust |
| REST ポート | 8210 |
| gRPC ポート | 9210 |
| DB | PostgreSQL（`project_master` スキーマ） |
| Kafka | プロデューサー（`k1s0.business.taskmanagement.projectmaster.project_type_changed.v1`, `k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1`） |
| イメージレジストリ | `harbor.internal.example.com/k1s0-business/project-master` |
| ビルドコンテキスト | `regions/business/taskmanagement` |

project-master-server はタスク管理領域のプロジェクトタイプ・ステータス定義・テナント拡張を管理するビジネスマスタサービスである。楽観的ロックとバージョニングによるデータ整合性と Outbox pattern による信頼性の高いイベント配信を実装する。

> **注記**: project-master-server は business tier に属するため、ビルドコンテキストは `regions/business/taskmanagement` となる。system ライブラリへの参照は相対パスで解決される。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm` + `cargo-chef`（依存キャッシュ 4 ステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| ビルドコマンド | `cargo build --release -p k1s0-project-master-server` |
| 公開ポート | 8210（REST API）、9210（gRPC） |
| 実行ユーザー | `nonroot:nonroot` |

### Dockerfile

```dockerfile
# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context must be ./regions/business/taskmanagement (to include library dependencies)
FROM rust:1.93-bookworm AS chef

# cargo-chef のインストール（依存キャッシュ用）
RUN cargo install cargo-chef --locked

# protobuf コンパイラ（tonic-build 用）と cmake + build-essential（rdkafka 用）のインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# planner ステージ: 依存情報レシピの生成
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# cook ステージ: 依存のみビルド（キャッシュ層）
FROM chef AS cook
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-project-master-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-project-master-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-project-master-server /k1s0-project-master-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8210 9210

ENTRYPOINT ["/k1s0-project-master-server"]
```

---

## 環境変数

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_NAME` | アプリケーション名 | `k1s0-project-master-server` |
| `APP_ENVIRONMENT` | 実行環境（`dev` / `staging` / `production`） | `dev` |
| `SERVER_HOST` | バインドアドレス | `0.0.0.0` |
| `SERVER_PORT` | REST API ポート | `8210` |
| `SERVER_GRPC_PORT` | gRPC ポート | `9210` |
| `DATABASE_URL` | PostgreSQL 接続文字列 | - |
| `DATABASE_SCHEMA` | スキーマ名 | `project_master` |
| `DATABASE_MAX_CONNECTIONS` | 最大接続数 | `25` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス（カンマ区切り） | - |
| `AUTH_JWKS_URL` | JWT 検証用 JWKS URL | - |
| `AUTH_ISSUER` | JWT issuer | - |
| `AUTH_AUDIENCE` | JWT audience | - |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry Collector エンドポイント | - |
| `RUST_LOG` | ログレベル | `info` |

### Vault シークレット

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/business/project-master/database` |
| Kafka SASL | `secret/data/k1s0/business/kafka/sasl` |

---

## DB マイグレーション

プロジェクトマスタデータは PostgreSQL の `project_master` スキーマに格納する。詳細スキーマは [database.md](database.md) 参照。

マイグレーションは `sqlx migrate run` で実行する。CI/CD パイプラインでは deploy ジョブの前に migration ジョブを実行し、スキーマの整合性を保証する。

### Docker Compose による DB 初期化

開発環境では `infra/docker/init-db/14-project-master-schema.sql` を PostgreSQL 起動時に自動実行してスキーマを初期化する。

```
migration → deploy-dev → deploy-staging → deploy-prod
```

---

## ヘルスチェック

| エンドポイント | パス | ポート | 用途 |
| --- | --- | --- | --- |
| Liveness | `/healthz` | 8210 | プロセス生存確認 |
| Readiness | `/readyz` | 8210 | DB 接続確認を含むトラフィック受付可否 |
| Metrics | `/metrics` | 8210 | Prometheus メトリクス |

### Kubernetes Probes

```yaml
# Liveness Probe: プロセスの生存確認
livenessProbe:
  httpGet:
    path: /healthz
    port: 8210
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe: DB 接続を含むトラフィック受付可否の確認
readinessProbe:
  httpGet:
    path: /readyz
    port: 8210
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

---

## リソース要件

### 本番環境

| リソース | requests | limits |
| --- | --- | --- |
| CPU | 200m | 500m |
| Memory | 256Mi | 512Mi |

### dev 環境

| リソース | requests | limits |
| --- | --- | --- |
| CPU | 100m | 250m |
| Memory | 128Mi | 256Mi |

---

## スケーリング設定

### HPA（Horizontal Pod Autoscaler）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| 最小レプリカ数 | 2 | 1 |
| 最大レプリカ数 | 8 | 1 |
| CPU 閾値 | 70% | - |
| Memory 閾値 | 80% | - |

### PDB（PodDisruptionBudget）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| minAvailable | 1 | - |

---

## Helm Chart

Helm Chart は `infra/helm/services/business/project-master/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-business/project-master
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8210
  grpcPort: 9210

service:
  type: ClusterIP
  port: 80
  grpcPort: 9210

resources:
  requests:
    cpu: 200m
    memory: 256Mi
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
  role: "business"
  secrets:
    - path: "secret/data/k1s0/business/project-master/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/business/kafka/sasl"
      key: "credentials"
      mountPath: "/vault/secrets/kafka-sasl"

kafka:
  enabled: true
  brokers: []

labels:
  tier: business
```

### dev 環境オーバーライド

```yaml
replicaCount: 1

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: false

pdb:
  enabled: false

vault:
  enabled: false

kafka:
  enabled: false
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

## ポートマッピング

| 環境 | ホストポート | コンテナポート | プロトコル |
| --- | --- | --- | --- |
| docker-compose | 8210 | 8210 | REST API |
| docker-compose | 9210 | 9210 | gRPC |
| Kubernetes | 80（Service） | 8210（Pod） | REST API |
| Kubernetes | 9210（Service） | 9210（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | プロジェクトタイプ・ステータス定義・テナント拡張の永続化 | Yes |
| Kafka | プロジェクトタイプ・ステータス定義変更イベント配信 | No（未設定時は NoopEventPublisher） |
| Keycloak | JWT 認証 | Yes |

---

## CI/CD パイプライン

### CI（`.github/workflows/project-master-ci.yaml`）

PR 時に `regions/business/taskmanagement/server/rust/project-master/**` の変更を検出してトリガーする。

```
detect-changes → lint-rust → test-rust → build-rust → security-scan
```

| ジョブ | 内容 |
| --- | --- |
| `detect-changes` | `dorny/paths-filter@v3` でパス変更検出 |
| `lint-rust` | `cargo fmt --check` + `cargo clippy -D warnings` |
| `test-rust` | `cargo test --all`（DB テストは `--features db-tests` で有効化） |
| `build-rust` | `cargo build --release` |
| `security-scan` | Trivy ファイルシステムスキャン（HIGH/CRITICAL） |

### CD（`.github/workflows/project-master-deploy.yaml`）

main ブランチへの push 時にトリガー。

```
build-and-push → migration → deploy-dev → deploy-staging → deploy-prod（手動承認）
```

| ジョブ | 内容 |
| --- | --- |
| `build-and-push` | Docker Buildx ビルド、Harbor プッシュ、Cosign 署名、Trivy イメージスキャン、SBOM 生成 |
| `migration` | `sqlx migrate run` で DB マイグレーション実行 |
| `deploy-dev` | Cosign 署名検証 → Helm upgrade（dev 環境） |
| `deploy-staging` | Cosign 署名検証 → Helm upgrade（staging 環境） |
| `deploy-prod` | Cosign 署名検証 → Helm upgrade（prod 環境、手動承認必須） |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
