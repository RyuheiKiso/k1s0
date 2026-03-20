# business-domain-master-server デプロイ仕様

## 概要

domain-master サーバーのデプロイは [共通デプロイ仕様](../_common/deploy.md) に従い、Helm Chart でデプロイする。

---

## ポート

| プロトコル | ポート | 用途 |
| --- | --- | --- |
| REST | 8210 | HTTP API |
| gRPC | 50061 | gRPC API |

---

## Dockerfile

マルチステージビルド（Rust builder → debian slim runtime）。

```dockerfile
# Stage 1: Build
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release --bin k1s0-domain-master-server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/k1s0-domain-master-server /usr/local/bin/
COPY --from=builder /app/config/ /etc/k1s0/domain-master/

EXPOSE 8210 50061

ENTRYPOINT ["k1s0-domain-master-server"]
```

---

## Helm Chart

k1s0 標準テンプレートを使用。

### values.yaml

```yaml
replicaCount: 2

image:
  repository: ghcr.io/k1s0-platform/domain-master-server
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  restPort: 8210
  grpcPort: 50061

resources:
  requests:
    cpu: 200m
    memory: 256Mi
  limits:
    cpu: 500m
    memory: 512Mi

readinessProbe:
  httpGet:
    path: /readyz
    port: 8210
  initialDelaySeconds: 5
  periodSeconds: 10

livenessProbe:
  httpGet:
    path: /healthz
    port: 8210
  initialDelaySeconds: 10
  periodSeconds: 30

env:
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: domain-master-db-secret
        key: url
  - name: KAFKA_BROKERS
    valueFrom:
      configMapKeyRef:
        name: kafka-config
        key: brokers
  - name: AUTH_JWKS_URL
    valueFrom:
      configMapKeyRef:
        name: auth-config
        key: jwks-url
```

---

## ヘルスチェック

| エンドポイント | 用途 | 成功条件 |
| --- | --- | --- |
| `GET /healthz` | Liveness Probe | HTTP 200 |
| `GET /readyz` | Readiness Probe | HTTP 200（DB 接続確認済み） |

---

## CI/CD

GitHub Actions を使用。

### ワークフロー

```yaml
name: domain-master-server

on:
  push:
    paths:
      - 'regions/business/accounting/server/rust/domain-master/**'
  pull_request:
    paths:
      - 'regions/business/accounting/server/rust/domain-master/**'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --manifest-path regions/business/accounting/server/rust/domain-master/Cargo.toml

  build-and-push:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v6
        with:
          context: ./regions/business/accounting
          file: ./regions/business/accounting/server/rust/domain-master/Dockerfile
          push: true
          tags: ghcr.io/k1s0-platform/domain-master-server:${{ github.sha }}

  deploy:
    needs: build-and-push
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          helm upgrade --install domain-master ./charts/domain-master \
            --set image.tag=${{ github.sha }} \
            --namespace accounting
```

---

## 環境変数

| 変数名 | 説明 | 例 |
| --- | --- | --- |
| `DATABASE_URL` | PostgreSQL 接続文字列 | `postgresql://k1s0:k1s0@postgres:5432/k1s0` |
| `KAFKA_BROKERS` | Kafka ブローカーアドレス | `kafka:9092` |
| `AUTH_JWKS_URL` | JWKS エンドポイント | `http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs` |
| `RUST_LOG` | ログレベル | `info,k1s0_domain_master_server=debug` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry エンドポイント | `http://otel-collector:4317` |

---

## Docker Compose (開発環境)

```yaml
domain-master-server:
  build:
    context: ./regions/business/accounting
    dockerfile: server/rust/domain-master/Dockerfile
  ports:
    - "8210:8210"
    - "9061:50061"
  environment:
    - DATABASE_URL=postgresql://k1s0:k1s0@postgres:5432/k1s0
    - KAFKA_BROKERS=kafka:9092
    - AUTH_JWKS_URL=http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs
    - RUST_LOG=info,k1s0_domain_master_server=debug
  depends_on:
    - postgres
    - kafka
    - keycloak
```
