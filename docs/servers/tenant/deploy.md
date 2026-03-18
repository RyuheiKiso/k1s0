# system-tenant-server デプロイ設計

system-tenant-server の Dockerfile・Helm values・デプロイ設定を定義する。概要・API 定義・アーキテクチャは [system-tenant-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | `k1s0-tenant-server` |
| Tier | system |
| 実装言語 | Rust |
| REST ポート | 8089 |
| gRPC ポート | 50051 |
| DB | PostgreSQL（`tenant` スキーマ） |
| Kafka | プロデューサー（`k1s0.system.tenant.events.v1`） |
| 外部連携 | Keycloak Admin API（realm プロビジョニング） |
| イメージレジストリ | `harbor.internal.example.com/k1s0-system/tenant` |
| ビルドコンテキスト | `regions/system` |

tenant-server はマルチテナンシーを実現するテナント管理サービスである。テナントプロビジョニング（Keycloak realm + DB スキーマ + 初期設定）を Saga パターンで実行し、テナントライフサイクル管理（作成・更新・一時停止・削除）とメンバー管理を提供する。

---

## コンテナイメージ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm` + `cargo-chef`（依存キャッシュ 4 ステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| busybox | ヘルスチェック用のシェルコマンド提供 |
| ビルドコマンド | `cargo build --release -p k1s0-tenant-server` |
| 公開ポート | 8089（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot` |

### Dockerfile

```dockerfile
# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context must be ./regions/system (to include library dependencies)
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-tenant-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-tenant-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-tenant-server /k1s0-tenant-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8089 50051

ENTRYPOINT ["/k1s0-tenant-server"]
```

### イメージタグ戦略

| タグ | 形式 |
| --- | --- |
| バージョン | `{VERSION}`（git describe から取得） |
| バージョン + SHA | `{VERSION}-{SHORT_SHA}` |
| latest | `latest` |

---

## 環境変数

tenant-server は `config.yaml` による設定を基本とし、環境変数でのオーバーライドをサポートする。

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_NAME` | アプリケーション名 | `k1s0-tenant-server` |
| `APP_ENVIRONMENT` | 実行環境（`dev` / `staging` / `production`） | `dev` |
| `SERVER_HOST` | バインドアドレス | `0.0.0.0` |
| `SERVER_HTTP_PORT` | REST API ポート | `8089` |
| `SERVER_GRPC_PORT` | gRPC ポート | `50051` |
| `DATABASE_HOST` | PostgreSQL ホスト | - |
| `DATABASE_PORT` | PostgreSQL ポート | `5432` |
| `DATABASE_NAME` | データベース名 | - |
| `DATABASE_USER` | DB 接続ユーザー | - |
| `DATABASE_PASSWORD` | DB パスワード（Vault から注入推奨） | - |
| `DATABASE_SSL_MODE` | SSL モード | `disable` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス（カンマ区切り） | - |
| `KEYCLOAK_BASE_URL` | Keycloak Admin API ベース URL | - |
| `KEYCLOAK_REALM` | Keycloak 管理用 realm | `master` |
| `KEYCLOAK_CLIENT_ID` | Keycloak Admin クライアント ID | `admin-cli` |
| `KEYCLOAK_CLIENT_SECRET` | Keycloak Admin クライアントシークレット（Vault から注入推奨） | - |
| `AUTH_JWKS_URL` | JWT 検証用 JWKS URL | - |
| `AUTH_ISSUER` | JWT issuer | - |
| `AUTH_AUDIENCE` | JWT audience | - |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry Collector エンドポイント | - |
| `RUST_LOG` | ログレベル | `info` |

### Vault シークレット

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/tenant/database` |
| Keycloak Admin Secret | `secret/data/k1s0/system/keycloak/admin` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

テナント情報・メンバーシップデータは PostgreSQL の `tenant` スキーマに格納する。テナントプロビジョニング時に Keycloak realm と DB スキーマの作成を Saga パターンで実行する。詳細スキーマは [database.md](database.md) 参照。

マイグレーションは `sqlx migrate run` で実行する。CI/CD パイプラインでは deploy ジョブの前に migration ジョブを実行し、スキーマの整合性を保証する。

```
migration → deploy-dev → deploy-staging → deploy-prod
```

---

## ヘルスチェック

| エンドポイント | パス | ポート | 用途 |
| --- | --- | --- | --- |
| Liveness | `/healthz` | 8089 | プロセス生存確認 |
| Readiness | `/readyz` | 8089 | PostgreSQL・Keycloak・Kafka 接続確認を含むトラフィック受付可否 |
| Metrics | `/metrics` | 8089 | Prometheus メトリクス |

### Kubernetes Probes

```yaml
# Liveness Probe: プロセスの生存確認
livenessProbe:
  httpGet:
    path: /healthz
    port: 8089
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe: PostgreSQL・Keycloak・Kafka 接続を含むトラフィック受付可否の確認
readinessProbe:
  httpGet:
    path: /readyz
    port: 8089
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

> tenant-server は Keycloak Admin API クライアント（reqwest）と Kafka プロデューサーを持ち、Saga パターンによるプロビジョニング処理で一時的にメモリを多く消費するため、余裕を持って確保する。

---

## スケーリング設定

### HPA（Horizontal Pod Autoscaler）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| 最小レプリカ数 | 2 | 1 |
| 最大レプリカ数 | 5 | 1 |
| CPU 閾値 | 70% | - |

### PDB（PodDisruptionBudget）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| minAvailable | 1 | - |

> テナントプロビジョニングは Saga パターンで実行されるため、処理中のインスタンスが中断されないよう PDB を設定する。

---

## Helm Chart

Helm Chart は `infra/helm/services/system/tenant/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/tenant
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8089
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

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
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

pdb:
  enabled: true
  minAvailable: 1

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/tenant/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/keycloak/admin"
      key: "client_secret"
      mountPath: "/vault/secrets/keycloak-secret"
    - path: "secret/data/k1s0/system/kafka/sasl"
      key: "credentials"
      mountPath: "/vault/secrets/kafka-sasl"

kafka:
  enabled: true
  brokers: []

labels:
  tier: system
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
| docker-compose | 8089 | 8089 | REST API |
| docker-compose | 50051 | 50051 | gRPC |
| Kubernetes | 80（Service） | 8089（Pod） | REST API |
| Kubernetes | 50051（Service） | 50051（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | テナント情報・メンバーシップ・プロビジョニングジョブの永続化 | Yes |
| Keycloak | realm プロビジョニング・ユーザー管理（Admin API） | Yes |
| Kafka | テナントイベント配信 | No（未設定時はイベント配信をスキップ） |
| saga-server | プロビジョニング Saga 連携 | No（saga-server 未接続時はローカル Saga） |

---

## Kong ルーティング

```yaml
services:
  - name: tenant-v1
    url: http://tenant-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: tenant-v1-route
        paths:
          - /api/v1/tenants
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## CI/CD パイプライン

### CI（`.github/workflows/tenant-ci.yaml`）

PR 時に `regions/system/server/rust/tenant/**` の変更を検出してトリガーする。

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

### CD（`.github/workflows/tenant-deploy.yaml`）

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

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト（ステータス遷移・バリデーション） | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/keycloak | 統合テスト | `wiremock`（Keycloak Admin API モック） |

---

## 関連ドキュメント

- [system-tenant-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
