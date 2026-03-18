# system-workflow-server デプロイ設計

system-workflow-server の Dockerfile・Helm values・デプロイ設定を定義する。概要・API 定義・アーキテクチャは [system-workflow-server.md](server.md) を参照。

---

## デプロイ概要

| 項目 | 値 |
| --- | --- |
| サーバー名 | `k1s0-workflow-server` |
| Tier | system |
| 実装言語 | Rust |
| REST ポート | 8100 |
| gRPC ポート | 50051 |
| DB | PostgreSQL（`workflow` スキーマ） |
| Kafka | プロデューサー（`k1s0.system.workflow.state_changed.v1`, `k1s0.system.notification.requested.v1`） |
| 外部連携 | scheduler-server（期日監視ジョブ登録）、notification-server（期日超過通知） |
| イメージレジストリ | `harbor.internal.example.com/k1s0-system/workflow` |
| ビルドコンテキスト | `regions/system` |

workflow-server は人間タスク・承認フロー込みのワークフローオーケストレーションサービスである。ワークフロー定義管理、インスタンス実行管理、人間タスクの承認/却下/再割当を提供し、scheduler-server と連携してタスク期日監視を行う。

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
| ビルドコマンド | `cargo build --release -p k1s0-workflow-server` |
| 公開ポート | 8100（REST API）、50051（gRPC） |
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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-workflow-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .
RUN cargo build --release -p k1s0-workflow-server

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-workflow-server /k1s0-workflow-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8100 50051

ENTRYPOINT ["/k1s0-workflow-server"]
```

### イメージタグ戦略

| タグ | 形式 |
| --- | --- |
| バージョン | `{VERSION}`（git describe から取得） |
| バージョン + SHA | `{VERSION}-{SHORT_SHA}` |
| latest | `latest` |

---

## 環境変数

workflow-server は `config.yaml` による設定を基本とし、環境変数でのオーバーライドをサポートする。

| 環境変数 | 説明 | デフォルト |
| --- | --- | --- |
| `APP_NAME` | アプリケーション名 | `k1s0-workflow-server` |
| `APP_ENVIRONMENT` | 実行環境（`dev` / `staging` / `production`） | `dev` |
| `SERVER_HOST` | バインドアドレス | `0.0.0.0` |
| `SERVER_PORT` | REST API ポート | `8100` |
| `SERVER_GRPC_PORT` | gRPC ポート | `50051` |
| `DATABASE_HOST` | PostgreSQL ホスト | - |
| `DATABASE_PORT` | PostgreSQL ポート | `5432` |
| `DATABASE_NAME` | データベース名 | - |
| `DATABASE_USER` | DB 接続ユーザー | - |
| `DATABASE_PASSWORD` | DB パスワード（Vault から注入推奨） | - |
| `DATABASE_SSL_MODE` | SSL モード | `disable` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス（カンマ区切り） | - |
| `KAFKA_STATE_TOPIC` | 状態変化イベントトピック | `k1s0.system.workflow.state_changed.v1` |
| `KAFKA_NOTIFICATION_TOPIC` | 通知要求トピック | `k1s0.system.notification.requested.v1` |
| `SCHEDULER_INTERNAL_ENDPOINT` | scheduler-server 内部 API エンドポイント | - |
| `OVERDUE_CHECK_CRON` | 期日超過チェック cron 式 | `*/15 * * * *` |
| `AUTH_JWKS_URL` | JWT 検証用 JWKS URL | - |
| `AUTH_ISSUER` | JWT issuer | - |
| `AUTH_AUDIENCE` | JWT audience | - |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry Collector エンドポイント | - |
| `RUST_LOG` | ログレベル | `info` |

### Vault シークレット

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/workflow/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## DB マイグレーション

ワークフロー定義・インスタンス・人間タスクは PostgreSQL の `workflow` スキーマに格納する。workflow_definitions、workflow_instances、workflow_tasks の 3 テーブルで構成される。詳細スキーマは [database.md](database.md) 参照。

マイグレーションは `sqlx migrate run` で実行する。CI/CD パイプラインでは deploy ジョブの前に migration ジョブを実行し、スキーマの整合性を保証する。

```
migration → deploy-dev → deploy-staging → deploy-prod
```

---

## ヘルスチェック

| エンドポイント | パス | ポート | 用途 |
| --- | --- | --- | --- |
| Liveness | `/healthz` | 8100 | プロセス生存確認 |
| Readiness | `/readyz` | 8100 | PostgreSQL・Kafka 接続確認を含むトラフィック受付可否 |
| Metrics | `/metrics` | 8100 | Prometheus メトリクス |

### Kubernetes Probes

```yaml
# Liveness Probe: プロセスの生存確認
livenessProbe:
  httpGet:
    path: /healthz
    port: 8100
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe: PostgreSQL・Kafka 接続を含むトラフィック受付可否の確認
readinessProbe:
  httpGet:
    path: /readyz
    port: 8100
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

> workflow-server は Kafka プロデューサーと reqwest クライアント（scheduler-server 連携）を持ち、ワークフロー状態機械の処理で一時的にメモリを消費するため、余裕を持って確保する。

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

> ワークフローインスタンスの状態遷移処理が中断されないよう PDB を設定する。期日超過チェックの内部エンドポイントは scheduler-server から定期呼び出しされるため、常にトラフィックを受け付けられるレプリカが必要。

---

## Helm Chart

Helm Chart は `infra/helm/services/system/workflow/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/workflow
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8100
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
    - path: "secret/data/k1s0/system/workflow/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
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
| docker-compose | 8100 | 8100 | REST API |
| docker-compose | 50051 | 50051 | gRPC |
| Kubernetes | 80（Service） | 8100（Pod） | REST API |
| Kubernetes | 50051（Service） | 50051（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | ワークフロー定義・インスタンス・タスクの永続化 | Yes |
| Kafka | 状態変化イベント配信・通知要求イベント | No（未設定時は Noop publisher） |
| scheduler-server | 期日超過チェックジョブの登録 | No（未設定時は手動チェックのみ） |
| notification-server | 期日超過タスクの通知配信（Kafka 経由） | No（通知不要時はスキップ） |
| Keycloak | JWT 認証 | Yes |

---

## Kong ルーティング

```yaml
services:
  - name: workflow-v1
    url: http://workflow-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: workflow-v1-workflows-route
        paths:
          - /api/v1/workflows
        strip_path: false
      - name: workflow-v1-instances-route
        paths:
          - /api/v1/instances
        strip_path: false
      - name: workflow-v1-tasks-route
        paths:
          - /api/v1/tasks
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## CI/CD パイプライン

### CI（`.github/workflows/workflow-ci.yaml`）

PR 時に `regions/system/server/rust/workflow/**` の変更を検出してトリガーする。

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

### CD（`.github/workflows/workflow-deploy.yaml`）

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
| domain/entity | 単体テスト（状態遷移・バリデーション） | `#[cfg(test)]` + `assert!` |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/messaging | 統合テスト | `mockall`（Kafka プロデューサー） |

---

## 関連ドキュメント

- [system-workflow-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [database.md](database.md) -- データベーススキーマ
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
- [system-scheduler-server.md](../scheduler/server.md) -- 期日監視ジョブ連携先
- [system-notification-server.md](../notification/server.md) -- 通知連携先
