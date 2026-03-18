# system-auth-server デプロイ設計

system-auth-server の Dockerfile・テスト・CI/CD パイプライン・設定ファイル・Helm values を定義する。概要・API 定義・アーキテクチャは [system-auth-server.md](server.md) を参照。データベースマイグレーション・テスト方針の詳細は [共通デプロイ設計](../_common/deploy.md) を参照。

---

## Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストは `regions/system`（ライブラリ依存解決のため）。

```dockerfile
# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.93-bookworm AS builder

# protobuf-compiler（proto 生成）、cmake + build-essential（rdkafka ビルド）をインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# システム全体のライブラリ依存を含めるためにディレクトリ全体をコピー
COPY . .

RUN cargo build --release -p k1s0-auth-server

# Runtime stage
FROM debian:bookworm-slim

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-auth-server /k1s0-auth-server

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/k1s0-auth-server"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm`（マルチステージビルド） |
| ランタイムステージ | `debian:bookworm-slim`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-auth-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

---

## 設定ファイル例

### config.docker.yaml

Docker 環境用の設定ファイル。`regions/system/server/rust/auth/config/config.docker.yaml` に配置。

```yaml
app:
  name: "auth"
  version: "0.1.0"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

database:
  host: postgres
  port: 5432
  name: k1s0_system
  user: dev
  password: dev
  ssl_mode: disable

kafka:
  brokers:
    - "kafka:9092"
  consumer_group: "auth.docker"
  security_protocol: "PLAINTEXT"
```

---

## CI/CD パイプライン

### CI（`.github/workflows/auth-ci.yaml`）

PR 時に `regions/system/server/rust/auth/**` の変更を検出してトリガーする。

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

### CD（`.github/workflows/auth-deploy.yaml`）

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

**レジストリ**: `harbor.internal.example.com/k1s0-system/auth`

---

## ポートマッピング

| 環境 | ホストポート | コンテナポート | プロトコル |
| --- | --- | --- | --- |
| docker-compose | 8080 | 8080 | REST API |
| docker-compose | 50051 | 50051 | gRPC |
| Kubernetes | 80（Service） | 8080（Pod） | REST API |
| Kubernetes | 50051（Service） | 50051（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | 監査ログの永続化 | Yes |
| Keycloak | JWT トークン発行・ユーザー管理 | Yes |
| Kafka | 認証イベント配信 | No（未設定時は配信をスキップ） |

---

## Helm Chart

Helm Chart は `infra/helm/services/system/auth/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Chart 構成

```
infra/helm/services/system/auth/
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
| DB パスワード | `secret/data/k1s0/system/auth/database` |
| OIDC 設定 | `secret/data/k1s0/system/auth/oidc` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/auth
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8080
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
    memory: 256Mi

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
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/auth/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/auth/oidc"
      key: "client_secret"
      mountPath: "/vault/secrets/oidc"

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
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: false

pdb:
  enabled: false

vault:
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

## Kong ルーティング

```yaml
services:
  - name: auth-v1
    url: http://auth-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: auth-v1-route
        paths:
          - /api/v1/auth
        strip_path: false
      - name: auth-v1-users-route
        paths:
          - /api/v1/users
        strip_path: false
      - name: auth-v1-audit-route
        paths:
          - /api/v1/audit
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
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe
readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

---

## 関連ドキュメント

- [system-auth-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [auth データベース設計](database.md) -- データベーススキーマ・マイグレーション
- [共通デプロイ設計](../_common/deploy.md) -- データベースマイグレーション・テスト方針
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
