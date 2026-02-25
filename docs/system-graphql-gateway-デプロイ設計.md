# system-graphql-gateway デプロイ設計

system-graphql-gateway の Dockerfile・Helm values・環境変数・ヘルスチェック・リソース制限を定義する。概要・API 定義・アーキテクチャは [system-graphql-gateway設計.md](system-graphql-gateway設計.md) を参照。

---

## Dockerfile

[Dockerイメージ戦略.md](Dockerイメージ戦略.md) のマルチステージビルドテンプレートおよび他の system サーバーの Dockerfile パターンに従う。

```dockerfile
# Build stage
# Note: build context must be ./regions/system (to include library dependencies)
FROM rust:1.88-bookworm AS builder

# Install protobuf compiler (for tonic-build in build.rs) and
# cmake + build-essential (for rdkafka cmake-build feature)
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire system directory to resolve path dependencies
COPY . .

RUN cargo build --release -p k1s0-graphql-gateway

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-graphql-gateway /k1s0-graphql-gateway

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-graphql-gateway"]
```

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.88-bookworm`（他サーバーと統一） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（他サーバーと統一、debian:bookworm-slim ではない） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-graphql-gateway`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST / GraphQL API のみ、gRPC なし） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

---

## 環境変数一覧

コンテナ起動時に参照する環境変数を定義する。設定ファイル（config.yaml）の値を環境変数でオーバーライドできる。

| 環境変数 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `CONFIG_PATH` | - | `config/config.yaml` | 設定ファイルのパス |
| `ENVIRONMENT` | - | `production` | 実行環境（`development` / `staging` / `production`） |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | `http://localhost:4317` | OpenTelemetry Collector エンドポイント |
| `RUST_LOG` | - | `info` | tracing フィルタ（例: `info,k1s0_graphql_gateway_server=debug`） |
| `SERVER_PORT` | - | `8080` | HTTP リスニングポート |

Vault Agent Injector によって `/vault/secrets/` に書き出されたシークレットは、設定ファイル内のパスで参照する。graphql-gateway 自体はシークレットを直接保持しない。

---

## Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。graphql-gateway 固有の values は以下の通り。

```yaml
# values-graphql-gateway.yaml
# infra/helm/services/system/graphql-gateway/values.yaml

app:
  name: graphql-gateway
  tier: system

image:
  registry: harbor.internal.example.com
  repository: k1s0-system/graphql-gateway
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8080

service:
  type: ClusterIP
  port: 80
  targetPort: 8080

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

# リソース制限
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

# ヘルスチェック
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

# グレースフルシャットダウン
terminationGracePeriodSeconds: 30

# ConfigMap マウント（config.yaml）
configMap:
  name: graphql-gateway-config
  mountPath: /app/config/config.yaml
  subPath: config.yaml

# 環境変数
env:
  - name: CONFIG_PATH
    value: /app/config/config.yaml
  - name: ENVIRONMENT
    valueFrom:
      fieldRef:
        fieldPath: metadata.labels['app.kubernetes.io/environment']
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://otel-collector.observability.svc.cluster.local:4317"

# Vault Agent Injector（graphql-gateway はシークレット不要のため無効）
vault:
  enabled: false

# Pod ラベル
podLabels:
  app: graphql-gateway
  tier: system
  component: bff

# アフィニティ（高可用性のため異なるノードに分散）
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app: graphql-gateway
          topologyKey: kubernetes.io/hostname
```

---

## ConfigMap

```yaml
# infra/helm/services/system/graphql-gateway/templates/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: graphql-gateway-config
  namespace: k1s0-system
data:
  config.yaml: |
    app:
      name: "graphql-gateway"
      version: "0.1.0"
      environment: "production"

    server:
      host: "0.0.0.0"
      port: 8080

    graphql:
      introspection: false
      playground: false
      max_depth: 10
      max_complexity: 1000

    auth:
      jwks_url: "http://auth-server.k1s0-system.svc.cluster.local/jwks"

    backends:
      tenant:
        address: "http://tenant-server.k1s0-system.svc.cluster.local:9090"
        timeout_ms: 3000
      featureflag:
        address: "http://featureflag-server.k1s0-system.svc.cluster.local:9090"
        timeout_ms: 3000
      config:
        address: "http://config-server.k1s0-system.svc.cluster.local:9090"
        timeout_ms: 3000

    observability:
      log:
        level: "info"
        format: "json"
      trace:
        enabled: true
        endpoint: "jaeger.observability.svc.cluster.local:4317"
        sample_rate: 1.0
      metrics:
        enabled: true
        path: "/metrics"
```

---

## ヘルスチェック

### GET /healthz（Liveness Probe）

サーバープロセスの生存確認。常に `200 OK` を返す。kubelet が定期的に呼び出し、失敗が続いた場合はコンテナを再起動する。

```json
HTTP/1.1 200 OK
Content-Type: application/json

{"status": "ok"}
```

### GET /readyz（Readiness Probe）

バックエンド gRPC サービスへの接続確認。全バックエンドが応答した場合に `200 OK` を返す。失敗時は `503 Service Unavailable` を返し、Service のエンドポイントから除外される。

```json
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "ready",
  "checks": {
    "tenant_grpc": "ok",
    "featureflag_grpc": "ok",
    "config_grpc": "ok"
  }
}
```

失敗時（一部バックエンドが応答しない場合）:

```json
HTTP/1.1 503 Service Unavailable
Content-Type: application/json

{
  "status": "not_ready",
  "checks": {
    "tenant_grpc": "ok",
    "featureflag_grpc": "error: connection refused",
    "config_grpc": "ok"
  }
}
```

### Kubernetes Probe 設定の根拠

| パラメータ | Liveness | Readiness | 説明 |
| --- | --- | --- | --- |
| `initialDelaySeconds` | 10 | 5 | Rust バイナリ起動は高速なため短めに設定 |
| `periodSeconds` | 10 | 5 | Readiness はより頻繁に確認してトラフィック遮断を早める |
| `timeoutSeconds` | 5 | 3 | gRPC 疎通チェックの上限 |
| `failureThreshold` | 3 | 3 | 一時的な遅延では再起動・除外しない |

---

## リソース制限

### 本番（requests / limits）

```yaml
resources:
  requests:
    cpu: 100m      # 通常時の GraphQL 処理に十分
    memory: 128Mi  # Rust バイナリ + gRPC チャンネル
  limits:
    cpu: 500m      # ピーク時のバースト処理
    memory: 256Mi  # DataLoader キャッシュ + ストリーミング
```

### ステージング（requests / limits）

```yaml
resources:
  requests:
    cpu: 50m
    memory: 64Mi
  limits:
    cpu: 200m
    memory: 128Mi
```

### HPA（Horizontal Pod Autoscaler）設定

```yaml
autoscaling:
  enabled: true
  minReplicas: 2       # 高可用性のため最低 2 Pod
  maxReplicas: 10      # 最大スケールアウト
  targetCPUUtilizationPercentage: 70
```

CPU 使用率 70% をスケールアウトの閾値とする。GraphQL ゲートウェイは I/O バウンドなため、CPU 閾値による HPA が効果的。

---

## Kong ルーティング

[API設計.md](API設計.md) の Kong ルーティング設計に従い、graphql-gateway を Kong に登録する。

```yaml
services:
  - name: graphql-gateway-v1
    url: http://graphql-gateway.k1s0-system.svc.cluster.local:80
    routes:
      - name: graphql-route
        paths:
          - /graphql
        strip_path: false
        methods:
          - POST
          - GET
      - name: graphql-ws-route
        paths:
          - /graphql/ws
        strip_path: false
        protocols:
          - http
          - https
    plugins:
      - name: rate-limiting
        config:
          minute: 1000
          policy: redis
      - name: cors
        config:
          origins:
            - "https://*.k1s0.example.com"
          methods:
            - POST
            - GET
            - OPTIONS
          headers:
            - Authorization
            - Content-Type
```

---

## CI/CD パイプライン

### Docker イメージビルド

```yaml
# .github/workflows/graphql-gateway.yaml（抜粋）
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Harbor
        uses: docker/login-action@v3
        with:
          registry: harbor.internal.example.com
          username: ${{ secrets.HARBOR_USERNAME }}
          password: ${{ secrets.HARBOR_PASSWORD }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: regions/system
          push: ${{ github.ref == 'refs/heads/main' }}
          tags: |
            harbor.internal.example.com/k1s0-system/graphql-gateway:${{ github.sha }}
            harbor.internal.example.com/k1s0-system/graphql-gateway:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Helm デプロイ

```yaml
  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to staging
        run: |
          helm upgrade --install graphql-gateway \
            infra/helm/services/system/graphql-gateway \
            --namespace k1s0-system \
            --set image.tag=${{ github.sha }} \
            --values infra/helm/services/system/graphql-gateway/values.yaml \
            --wait --timeout 5m
```

---

## 関連ドキュメント

- [system-graphql-gateway設計.md](system-graphql-gateway設計.md) -- 概要・API 定義・アーキテクチャ
- [system-graphql-gateway-実装設計.md](system-graphql-gateway-実装設計.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [API設計.md](API設計.md) -- Kong ルーティング設計
