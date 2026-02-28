# system-dlq-manager-server デプロイ設計ガイド

> **仕様**: テーブル定義・APIスキーマは [deploy.md](./deploy.md) を参照。

---

## Dockerfile

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

RUN cargo build --release -p k1s0-dlq-manager

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /app/target/release/k1s0-dlq-manager /k1s0-dlq-manager

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/k1s0-dlq-manager"]
```

---

## 設定ファイル例

### config.docker.yaml

Docker 環境用の設定ファイル。`regions/system/server/rust/dlq-manager/config/config.docker.yaml` に配置。

```yaml
app:
  name: "dlq-manager"
  version: "0.1.0"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8084          # docker-compose 内部ポート（ホスト: 8086 → コンテナ: 8080）

database:
  host: postgres
  port: 5432
  name: dlq_db
  user: dev
  password: dev
  ssl_mode: disable

kafka:
  brokers:
    - "kafka:9092"
  consumer_group: "dlq-manager.docker"
  dlq_topic_pattern: "*.dlq.v1"
  security_protocol: "PLAINTEXT"
```

---

## Helm values（デフォルト）

```yaml
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/dlq-manager
  tag: ""
  pullPolicy: IfNotPresent

replicaCount: 2

container:
  port: 8080
  grpcPort: null        # REST API のみ

service:
  type: ClusterIP
  port: 80
  grpcPort: null

resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi

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
    - path: "secret/data/k1s0/system/dlq-manager/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"

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
  - name: dlq-manager-v1
    url: http://dlq-manager.k1s0-system.svc.cluster.local:80
    routes:
      - name: dlq-manager-v1-route
        paths:
          - /api/v1/dlq
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

### docker-compose 環境

distroless コンテナには curl/sh が含まれないため、`CMD-SHELL` によるヘルスチェックは使用不可。ホスト側から `curl` で確認する。

```bash
# ヘルスチェック
curl -f http://localhost:8086/healthz

# レディネスチェック
curl -f http://localhost:8086/readyz
```
