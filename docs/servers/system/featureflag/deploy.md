# system-featureflag-server デプロイ設計

system-featureflag-server の Dockerfile・テスト・CI/CD パイプライン・設定ファイル・Helm values を定義する。概要・API 定義・アーキテクチャは [system-featureflag-server.md](server.md) を参照。

---

## Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストは `regions/system`（ライブラリ依存解決のため）。cargo-chef による依存キャッシュを活用した 4 ステージビルド。

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
RUN cargo chef cook --release --recipe-path recipe.json -p k1s0-featureflag-server

# builder ステージ: ソースコードのビルド
FROM cook AS builder
COPY . .

ARG CARGO_FEATURES=""
RUN if [ -n "${CARGO_FEATURES}" ]; then \
      cargo build --release -p k1s0-featureflag-server --features "${CARGO_FEATURES}"; \
    else \
      cargo build --release -p k1s0-featureflag-server; \
    fi

FROM busybox:1.36.1-musl AS busybox

# runtime ステージ: 最小実行環境
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-featureflag-server /k1s0-featureflag-server
COPY --from=busybox /bin/busybox /busybox

USER nonroot:nonroot
EXPOSE 8087 50056

ENTRYPOINT ["/k1s0-featureflag-server"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.93-bookworm` + `cargo-chef`（依存キャッシュ 4 ステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| busybox | ヘルスチェック用のシェルコマンド提供 |
| ビルドコマンド | `cargo build --release -p k1s0-featureflag-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8087（REST API）、50056（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

---

## 設定ファイル例

### config.yaml

開発環境用の設定ファイル。`regions/system/server/rust/featureflag/config/config.yaml` に配置。

```yaml
app:
  name: k1s0-featureflag-server
  version: "0.1.0"
  environment: dev

server:
  host: "0.0.0.0"
  port: 8087
  grpc_port: 50051

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

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |

---

## CI/CD パイプライン

### CI（`.github/workflows/featureflag-ci.yaml`）

PR 時に `regions/system/server/rust/featureflag/**` の変更を検出してトリガーする。

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

### CD（`.github/workflows/featureflag-deploy.yaml`）

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

**レジストリ**: `harbor.internal.example.com/k1s0-system/featureflag`

---

## ポートマッピング

| 環境 | ホストポート | コンテナポート | プロトコル |
| --- | --- | --- | --- |
| docker-compose | 8187 | 8080 | REST API（8087 は NAT 競合のためデフォルトを 8187 に変更済み。`FEATUREFLAG_REST_HOST_PORT` 環境変数で上書き可能） |
| docker-compose | 50056 | 50051 | gRPC |
| Kubernetes | 80（Service） | 8087（Pod） | REST API |
| Kubernetes | 50056（Service） | 50056（Pod） | gRPC |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | フラグ定義・バリアント・ルール・変更監査ログの永続化 | Yes |
| Kafka | フラグ変更通知（`k1s0.system.featureflag.changed.v1`） | Yes |

---

## Helm Chart

Helm Chart は `infra/helm/services/system/featureflag/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Chart 構成

```
infra/helm/services/system/featureflag/
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
| DB パスワード | `secret/data/k1s0/system/featureflag/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

### Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/featureflag
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8087
  grpcPort: 50056

service:
  type: ClusterIP
  port: 80
  grpcPort: 50056

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
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

pdb:
  enabled: true
  minAvailable: 1

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/featureflag/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/kafka/sasl"
      key: "password"
      mountPath: "/vault/secrets/kafka-sasl"

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
  - name: featureflag-v1
    url: http://featureflag.k1s0-system.svc.cluster.local:80
    routes:
      - name: featureflag-v1-flags-route
        paths:
          - /api/v1/flags
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
    port: 8087
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3

# Readiness Probe
readinessProbe:
  httpGet:
    path: /readyz
    port: 8087
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3
```

---

## 関連ドキュメント

- [system-featureflag-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [featureflag データベース設計](database.md) -- データベーススキーマ・マイグレーション
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
