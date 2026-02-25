# docker-compose システムサービス設計

docker-compose における auth-server・config-server・System プロファイルの詳細設定を定義する。基本方針・プロファイル設計は [docker-compose設計.md](docker-compose設計.md) を参照。

---

## System プロファイル サービス定義

system 層のアプリケーションサーバーは `docker-compose.override.yaml` で管理する。以下に各サービスの詳細設定を示す。

### auth-server（Rust 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `auth-rust` |
| ビルドコンテキスト | `./regions/system/server/rust/auth` |
| Dockerfile | マルチステージビルド（`rust:1.82-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8083:8080` / gRPC `50052:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（started） |
| 環境変数 | `CONFIG_PATH=/app/config/config.dev.yaml` |
| ボリューム | `./regions/system/server/rust/auth/config:/app/config` |

```yaml
auth-rust:
  build:
    context: ./regions/system/server/rust/auth
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8083:8080"    # REST
    - "50052:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.dev.yaml
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_started
  volumes:
    - ./regions/system/server/rust/auth/config:/app/config
  healthcheck:
    test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
    interval: 10s
    timeout: 5s
    retries: 5
```

### config-server（Rust 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `config-rust` |
| ビルドコンテキスト | `./regions/system/server/rust/config` |
| Dockerfile | マルチステージビルド（`rust:1.82-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8084:8080` / gRPC `50054:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（started） |
| 環境変数 | `CONFIG_PATH=/app/config/config.dev.yaml` |
| ボリューム | `./regions/system/server/rust/config/config:/app/config` |

```yaml
config-rust:
  build:
    context: ./regions/system/server/rust/config
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8084:8080"    # REST
    - "50054:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.dev.yaml
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_started
  volumes:
    - ./regions/system/server/rust/config/config:/app/config
  healthcheck:
    test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
    interval: 10s
    timeout: 5s
    retries: 5
```

### saga-server（Rust 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `saga-rust` |
| ビルドコンテキスト | `./regions/system/server/rust/saga` |
| Dockerfile | マルチステージビルド（`rust:1.82-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8085:8080` / gRPC `50055:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（started） |
| 環境変数 | `CONFIG_PATH=/app/config/config.dev.yaml` |
| ボリューム | `./regions/system/server/rust/saga/config:/app/config` |

```yaml
saga-rust:
  build:
    context: ./regions/system/server/rust/saga
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8085:8080"    # REST
    - "50055:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.dev.yaml
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_started
  volumes:
    - ./regions/system/server/rust/saga/config:/app/config
  healthcheck:
    test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
    interval: 10s
    timeout: 5s
    retries: 5
```

### dlq-manager（Rust 版）

DLQ（Dead Letter Queue）メッセージの管理・再処理を担う REST API サーバー。gRPC は提供しない。

| 項目 | 設定 |
| --- | --- |
| サービス名 | `dlq-manager` |
| ビルドコンテキスト | `./regions/system/server/rust/dlq-manager` |
| Dockerfile | マルチステージビルド（`rust:1.82-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8086:8080`（gRPC なし） |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy） |
| 環境変数 | `CONFIG_PATH=/app/config/config.docker.yaml` |
| ボリューム | `./regions/system/server/rust/dlq-manager/config:/app/config` |

```yaml
dlq-manager:
  build:
    context: ./regions/system/server/rust/dlq-manager
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8086:8080"    # REST のみ（gRPC なし）
  environment:
    - CONFIG_PATH=/app/config/config.docker.yaml
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
  volumes:
    - ./regions/system/server/rust/dlq-manager/config:/app/config
  healthcheck:
    test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
    interval: 10s
    timeout: 5s
    retries: 5
```

### ポート割り当て一覧

| サービス | REST ポート | gRPC ポート | 備考 |
| --- | --- | --- | --- |
| auth-rust | 8083 | 50052 | Rust 版 auth-server |
| config-rust | 8084 | 50054 | Rust 版 config-server |
| saga-rust | 8085 | 50055 | Rust 版 saga-server |
| dlq-manager | 8086 | - | DLQ 管理サーバー（gRPC なし） |

## アプリケーションサービスの追加

本ファイル（`docker-compose.yaml`）にはインフラサービスのみを定義する。アプリケーションサービスは `docker-compose.override.yaml` で管理し、各開発者がローカルで必要なサービスのみを起動できるようにする。

### 方式

- リポジトリに `docker-compose.override.yaml.example` を配置し、テンプレートとして提供する
- 各開発者は `docker-compose.override.yaml.example` をコピーして `docker-compose.override.yaml` を作成する
- `docker-compose.override.yaml` は `.gitignore` に追加し、各開発者のローカル設定として管理する
- Docker Compose は `docker-compose.yaml` と `docker-compose.override.yaml` を自動的にマージする

### docker-compose.override.yaml.example

各サービスの詳細設定は上記セクションを参照。

```yaml
# docker-compose.override.yaml.example
# このファイルを docker-compose.override.yaml にコピーして使用してください。
# 必要なサービスのコメントを解除して起動してください。

services:
  # --- system 層 ---
  # ポート割り当て:
  #   auth-rust:   REST 8083, gRPC 50052
  #   config-rust: REST 8084, gRPC 50054
  #   saga-rust:   REST 8085, gRPC 50055
  #   dlq-manager: REST 8086  (gRPC なし)
  #
  # auth-rust:
  #   build:
  #     context: ./regions/system/server/rust/auth
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8083:8080"
  #     - "50052:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.dev.yaml
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_started
  #   volumes:
  #     - ./regions/system/server/rust/auth/config:/app/config
  #   healthcheck:
  #     test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  #     interval: 10s
  #     timeout: 5s
  #     retries: 5
  #
  # config-rust:
  #   build:
  #     context: ./regions/system/server/rust/config
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8084:8080"
  #     - "50054:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.dev.yaml
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_started
  #   volumes:
  #     - ./regions/system/server/rust/config/config:/app/config
  #   healthcheck:
  #     test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  #     interval: 10s
  #     timeout: 5s
  #     retries: 5
  #
  # saga-rust:
  #   build:
  #     context: ./regions/system/server/rust/saga
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8085:8080"
  #     - "50055:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.dev.yaml
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_started
  #   volumes:
  #     - ./regions/system/server/rust/saga/config:/app/config
  #   healthcheck:
  #     test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  #     interval: 10s
  #     timeout: 5s
  #     retries: 5
  #
  # dlq-manager:
  #   build:
  #     context: ./regions/system/server/rust/dlq-manager
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8086:8080"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.docker.yaml
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #   volumes:
  #     - ./regions/system/server/rust/dlq-manager/config:/app/config
  #   healthcheck:
  #     test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  #     interval: 10s
  #     timeout: 5s
  #     retries: 5

  # --- service 層 ---
  # order-server:
  #   build:
  #     context: ./regions/service/order/server/rust
  #     dockerfile: Dockerfile
  #   profiles: [service]
  #   ports:
  #     - "8082:8080"
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #   volumes:
  #     - ./regions/service/order/server/rust/config:/app/config

  # --- API Gateway ---
  # NOTE: Kong は docker-compose.yaml 本体に定義済み（infra プロファイル）。
  # ここでは環境変数やポートのオーバーライドが必要な場合のみ定義する。
```

## ローカル環境のサービス名解決（D-102）

### 課題

本番環境（Kubernetes）では `postgres.k1s0-system.svc.cluster.local` のような DNS 名でサービスにアクセスするが、ローカル開発環境（docker-compose）には Kubernetes DNS が存在しない。

### 方針: docker-compose サービス名をそのまま使用する

ローカル環境では docker-compose の **サービス名**（`postgres`, `redis`, `kafka` 等）がコンテナ間 DNS として機能する。アプリケーションは `config/config.dev.yaml` でローカル用のホスト名を指定する。

### config.yaml との対応

| 接続先       | 本番（Kubernetes DNS）                                    | ローカル（docker-compose サービス名） |
| ------------ | --------------------------------------------------------- | ------------------------------------- |
| PostgreSQL   | `postgres.{k1s0-system\|k1s0-business\|k1s0-service}.svc.cluster.local` | `postgres`                 |
| MySQL        | `mysql.k1s0-system.svc.cluster.local`                    | `mysql`                               |
| Redis        | `redis.k1s0-system.svc.cluster.local`                    | `redis`                               |
| Kafka        | `kafka-0.messaging.svc.cluster.local`                    | `kafka`                               |
| Schema Registry | `schema-registry.k1s0-system.svc.cluster.local`      | `schema-registry`                     |
| Jaeger       | `jaeger.observability.svc.cluster.local`                 | `jaeger`                              |
| Vault        | `vault.k1s0-system.svc.cluster.local`                   | `vault`                               |
| Keycloak     | `keycloak.k1s0-system.svc.cluster.local`                | `keycloak`                            |
| Redis（BFF セッション用） | `redis-session.k1s0-system.svc.cluster.local` | `redis-session`                       |
| 他サービス   | `{service}.{namespace}.svc.cluster.local`                | `{docker-compose サービス名}`        |

> **注記**: Kubernetes 環境では kafka-1, kafka-2 等の追加ブローカーが存在するが、ブローカー台数は環境により異なるため代表例として kafka-0 のみ記載している。

### config.dev.yaml の例

```yaml
# config/config.dev.yaml（ローカル開発用）
database:
  host: "postgres"
  port: 5432
  user: "dev"
  password: "dev"
  ssl_mode: "disable"

kafka:
  brokers:
    - "kafka:9092"

redis:
  host: "redis"
  port: 6379

auth:
  jwt:
    issuer: "http://keycloak:8080/realms/k1s0"

observability:
  trace:
    endpoint: "jaeger:4317"
```

### ルール

- `config.yaml`（デフォルト）には **本番環境の DNS 名** を記載する
- `config.dev.yaml` に **docker-compose サービス名** で上書きする
- アプリケーションコード内でホスト名をハードコードしてはならない（すべて config.yaml 経由）
- docker-compose の `networks.default.name: k1s0-network` により、全コンテナが同一ネットワークに接続され名前解決が可能

## Kong API Gateway（ローカル開発用）

ローカル開発環境では Kong を DB-less モード（declarative config）で起動する。本番環境（DB-backed モード）とは異なり、設定ファイルを直接マウントする簡易構成とする。

> **注記**: 本番環境では Kong は DB-backed モード（PostgreSQL）で運用し、decK で宣言的に管理する。詳細は [APIゲートウェイ設計](APIゲートウェイ設計.md) を参照。

### docker-compose.yaml での定義

Kong は `docker-compose.yaml` 本体に `infra` プロファイルとして定義する。ローカル開発用の declarative config（`infra/kong/kong.dev.yaml`）をマウントする。

```yaml
kong:
  image: kong:3.8
  profiles: [infra]
  environment:
    KONG_DATABASE: "off"
    KONG_DECLARATIVE_CONFIG: /etc/kong/kong.yaml
    KONG_PROXY_LISTEN: "0.0.0.0:8000"
    KONG_ADMIN_LISTEN: "0.0.0.0:8001"
    KONG_PROXY_ACCESS_LOG: /dev/stdout
    KONG_ADMIN_ACCESS_LOG: /dev/stdout
    KONG_PROXY_ERROR_LOG: /dev/stderr
    KONG_ADMIN_ERROR_LOG: /dev/stderr
    KONG_LOG_LEVEL: info
  ports:
    - "8000:8000"    # Proxy
    - "8001:8001"    # Admin API
  volumes:
    - ./infra/kong/kong.dev.yaml:/etc/kong/kong.yaml:ro
  healthcheck:
    test: ["CMD", "kong", "health"]
    interval: 10s
    timeout: 5s
    retries: 3
```

### kong.yaml と kong.dev.yaml の使い分け

| ファイル | 用途 | サービス URL | rate-limiting | JWT |
| --- | --- | --- | --- | --- |
| `infra/kong/kong.yaml` | 本番用 decK 設定 | Kubernetes DNS | Redis policy | 有効 |
| `infra/kong/kong.dev.yaml` | ローカル開発用 declarative config | docker-compose サービス名 | local policy | 無効 |

### ローカル用 kong.dev.yaml

```yaml
# infra/kong/kong.dev.yaml（ローカル開発用 declarative config）
_format_version: "3.0"

services:
  - name: auth-v1
    url: http://auth-rust:8080
    routes:
      - name: auth-token-route
        paths:
          - /api/v1/auth
        strip_path: false
      - name: auth-users-route
        paths:
          - /api/v1/users
        strip_path: false
      - name: auth-audit-route
        paths:
          - /api/v1/audit
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 1000
          policy: local
      - name: request-size-limiting
        config:
          allowed_payload_size: 10

  - name: config-v1
    url: http://config-rust:8080
    routes:
      - name: config-route
        paths:
          - /api/v1/config
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 2000
          policy: local

  - name: auth-health
    url: http://auth-rust:8080
    routes:
      - name: auth-health-check
        paths:
          - /healthz
          - /readyz
        strip_path: false

plugins:
  - name: cors
    config:
      origins:
        - "http://localhost:3000"
      methods:
        - GET
        - POST
        - PUT
        - PATCH
        - DELETE
        - OPTIONS
      headers:
        - Authorization
        - Content-Type
        - X-Request-ID
      exposed_headers:
        - X-RateLimit-Limit
        - X-RateLimit-Remaining
        - X-RateLimit-Reset
      credentials: true
      max_age: 3600

  - name: rate-limiting
    config:
      minute: 5000
      policy: local
      fault_tolerant: true
      hide_client_headers: false

  - name: prometheus
    config:
      per_consumer: true
      status_code_metrics: true
      latency_metrics: true
      bandwidth_metrics: true
```

> **注記**: ローカル開発環境では JWT プラグインは無効にしている（開発効率を優先）。JWT 検証を含めたい場合は、`kong.dev.yaml` に `jwt` プラグインを追加すること。

## マルチステージビルド Dockerfile 設計

各サーバーの Dockerfile はマルチステージビルドで最小限のランタイムイメージを生成する。詳細は [Dockerイメージ戦略](Dockerイメージ戦略.md) を参照。

### Rust サーバー

```dockerfile
# regions/system/server/rust/auth/Dockerfile
# Build stage
FROM rust:1.82-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/k1s0-auth-server /k1s0-auth-server
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/k1s0-auth-server"]
```

| 項目 | 設定 |
| --- | --- |
| ビルドイメージ | `rust:1.82-bookworm` |
| ランタイムイメージ | `gcr.io/distroless/cc-debian12:nonroot`（C/C++ ランタイム含む） |
| 追加パッケージ | `protobuf-compiler`（tonic-build）, `cmake` + `build-essential`（rdkafka） |
| 依存キャッシュ | ダミー `main.rs` で依存クレートを先にビルド |
| 実行ユーザー | `nonroot:nonroot` |

### ヘルスチェック

全サーバーは REST エンドポイントでヘルスチェックを提供する。

| エンドポイント | 用途 | 検証対象 |
| --- | --- | --- |
| `GET /healthz` | Liveness | サーバープロセスの生存確認 |
| `GET /readyz` | Readiness | 依存サービス（DB, Keycloak, Kafka）への接続確認 |
| `GET /metrics` | Prometheus | メトリクスの公開 |

Docker Compose のヘルスチェックでは `/healthz` を使用する。

```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  interval: 10s
  timeout: 5s
  retries: 5
```

> **注記**: distroless イメージには `curl` が含まれないため、ビルドステージで `curl` バイナリをコピーするか、自前のヘルスチェックバイナリを使用する。ローカル開発環境では Docker Compose の `healthcheck` ではなく `depends_on` の条件で起動順序を制御するため、distroless 内の curl 有無は問題にならない。

---

## 関連ドキュメント

- [docker-compose設計.md](docker-compose設計.md) -- 基本方針・プロファイル設計
- [docker-compose-インフラサービス設計.md](docker-compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis・Kong の詳細設定
- [docker-compose-可観測性サービス設計.md](docker-compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定
- [system-server設計.md](system-server設計.md) -- 認証サーバー設計
- [system-config-server設計.md](system-config-server設計.md) -- 設定管理サーバー設計
- [system-saga-server設計.md](system-saga-server設計.md) -- Saga オーケストレーションサーバー設計
- [system-dlq-manager-server設計.md](system-dlq-manager-server設計.md) -- DLQ 管理サーバー設計
- [APIゲートウェイ設計.md](APIゲートウェイ設計.md) -- Kong 構成管理
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
