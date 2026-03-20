# service-order-server デプロイ設計

service-order-server の Dockerfile・Helm values・デプロイ設定を定義する。概要・API 定義・アーキテクチャは [service-order-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | `k1s0-order-server` |
| Tier | service |
| 実装言語 | Rust |
| REST ポート | 8310 |
| gRPC ポート | なし |
| DB | PostgreSQL（`order_service` スキーマ） |
| Kafka | プロデューサー（`k1s0.service.order.created.v1`, `k1s0.service.order.updated.v1`, `k1s0.service.order.cancelled.v1`） |
| イメージレジストリ | `harbor.internal.example.com/k1s0-service/order` |
| ビルドコンテキスト | `regions/service` |

order-server は顧客の注文作成・照会・ステータス管理を REST API で提供し、注文イベントを Kafka に非同期配信するサービスである。楽観的ロックによる排他制御とステートマシンベースのステータス遷移を実装する。

> **注記**: order-server は service tier に属するため、ビルドコンテキストは `regions/service` となる。system ライブラリへの参照は相対パスで解決される。

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
| ビルドコマンド | `cargo build --release -p k1s0-order-server` |
| 公開ポート | 8310（REST API） |
| 実行ユーザー | `nonroot:nonroot` |

### Dockerfile

```dockerfile
# chef ベースステージ: cargo-chef とビルド依存のインストール
# Note: build context must be ./regions/service (to include library dependencies)
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-order-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-order-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-order-server /k1s0-order-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8310

ENTRYPOINT ["/k1s0-order-server"]
```

### イメージタグ戦略

| タグ | 形式 |
| --- | --- |
| バージョン | `{VERSION}`（git describe から取得） |
| バージョン + SHA | `{VERSION}-{SHORT_SHA}` |
| latest | `latest` |

---

## 環境変数

order-server は `config.yaml` による設定を基本とし、環境変数でのオーバーライドをサポートする。

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_NAME` | アプリケーション名 | `k1s0-order-server` |
| `APP_ENVIRONMENT` | 実行環境（`dev` / `staging` / `production`） | `dev` |
| `SERVER_HOST` | バインドアドレス | `0.0.0.0` |
| `SERVER_PORT` | REST API ポート | `8310` |
| `DATABASE_HOST` | PostgreSQL ホスト | - |
| `DATABASE_PORT` | PostgreSQL ポート | `5432` |
| `DATABASE_NAME` | データベース名 | - |
| `DATABASE_SCHEMA` | スキーマ名 | `order_service` |
| `DATABASE_USER` | DB 接続ユーザー | - |
| `DATABASE_PASSWORD` | DB パスワード（Vault から注入推奨） | - |
| `DATABASE_SSL_MODE` | SSL モード | `disable` |
| `DATABASE_MAX_CONNECTIONS` | 最大接続数 | `25` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス（カンマ区切り） | - |
| `AUTH_JWKS_URL` | JWT 検証用 JWKS URL | - |
| `AUTH_ISSUER` | JWT issuer | - |
| `AUTH_AUDIENCE` | JWT audience | - |
| `AUTH_JWKS_CACHE_TTL_SECS` | JWKS キャッシュ TTL（秒） | `300` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry Collector エンドポイント | - |
| `RUST_LOG` | ログレベル | `info` |

### Vault シークレット

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/service/order/database` |
| Kafka SASL | `secret/data/k1s0/service/kafka/sasl` |

---

## DB マイグレーション

注文データは PostgreSQL の `order_service` スキーマに格納する。orders テーブルと order_items テーブルで構成される。詳細スキーマは [database.md](database.md) 参照。

マイグレーションは `sqlx migrate run` で実行する。CI/CD パイプラインでは deploy ジョブの前に migration ジョブを実行し、スキーマの整合性を保証する。

```
migration → deploy-dev → deploy-staging → deploy-prod
```

---

## ヘルスチェック

| エンドポイント | パス | ポート | 用途 |
| --- | --- | --- | --- |
| Liveness | `/healthz` | 8310 | プロセス生存確認 |
| Readiness | `/readyz` | 8310 | DB 接続確認を含むトラフィック受付可否 |
| Metrics | `/metrics` | 8310 | Prometheus メトリクス |

### Kubernetes Probes

```yaml
# Liveness Probe: プロセスの生存確認
livenessProbe:
  httpGet:
    path: /healthz
    port: 8310
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe: DB 接続を含むトラフィック受付可否の確認
readinessProbe:
  httpGet:
    path: /readyz
    port: 8310
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

> order-server は DB 接続プールと Kafka プロデューサーを持つため、メモリを標準的に確保する。

---

## スケーリング設定

### HPA（Horizontal Pod Autoscaler）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| 最小レプリカ数 | 2 | 1 |
| 最大レプリカ数 | 10 | 1 |
| CPU 閾値 | 70% | - |
| Memory 閾値 | 80% | - |

### PDB（PodDisruptionBudget）

| 項目 | 本番 | dev |
| --- | --- | --- |
| 有効 | true | false |
| minAvailable | 1 | - |

> 注文処理はトランザクション性が重要なため、maxReplicas を 10 に設定し、ピーク時の負荷に対応する。

---

## Helm Chart

Helm Chart は `infra/helm/services/service/order/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-service/order
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8310

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
  targetMemoryUtilizationPercentage: 80

pdb:
  enabled: true
  minAvailable: 1

vault:
  enabled: true
  role: "service"
  secrets:
    - path: "secret/data/k1s0/service/order/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/service/kafka/sasl"
      key: "credentials"
      mountPath: "/vault/secrets/kafka-sasl"

kafka:
  enabled: true
  brokers: []

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
| docker-compose | 8310 | 8310 | REST API |
| Kubernetes | 80（Service） | 8310（Pod） | REST API |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | 注文・注文明細の永続化 | Yes |
| Kafka | 注文イベント配信 | No（未設定時は NoopOrderEventPublisher） |
| Keycloak | JWT 認証 | Yes |

---

## Kong ルーティング

```yaml
services:
  - name: order-v1
    url: http://order-server.k1s0-service.svc.cluster.local:80
    routes:
      - name: order-v1-route
        paths:
          - /api/v1/orders
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## CI/CD パイプライン

### CI（`.github/workflows/order-ci.yaml`）

PR 時に `regions/service/order/server/rust/order/**` の変更を検出してトリガーする。

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

### CD（`.github/workflows/order-deploy.yaml`）

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
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP） | `axum-test` + `tokio::test` |
| infrastructure/database | 統合テスト（DB） | `testcontainers`（`db-tests` feature flag） |
| infrastructure/kafka | 統合テスト | `mockall` + `rdkafka` |

---

## 関連ドキュメント

- [service-order-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [implementation.md](implementation.md) -- Rust 実装詳細
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
