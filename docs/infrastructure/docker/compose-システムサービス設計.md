# docker-compose システムサービス設計

docker-compose における auth-server・config-server・System プロファイルの詳細設定を定義する。基本方針・プロファイル設計は [docker-compose設計.md](docker-compose設計.md) を参照。

---

## System プロファイル サービス定義

system tier のアプリケーションサーバーは `docker-compose.dev.yaml`（開発用オーバーライド）で管理する。以下に各サービスの詳細設定を示す。

### auth-server（Rust 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `auth-rust` |
| ビルドコンテキスト | `./regions/system`（dockerfile: `./server/rust/auth/Dockerfile`） |
| Dockerfile | マルチステージビルド（`rust:1.93-bookworm` → `debian:bookworm-slim`） |
| プロファイル | `system` |
| ポート | REST `${AUTH_REST_HOST_PORT:-8083}:8080` / gRPC `${AUTH_GRPC_HOST_PORT:-50052}:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（healthy） |
| 環境変数 | `CONFIG_PATH=/app/config/config.docker.yaml`, `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317` |
| ボリューム | `./regions/system/server/rust/auth/config:/app/config` |

```yaml
auth-rust:
  build:
    context: ./regions/system
    dockerfile: ./server/rust/auth/Dockerfile
  profiles: [system]
  ports:
    - "${AUTH_REST_HOST_PORT:-8083}:8080"    # REST
    - "${AUTH_GRPC_HOST_PORT:-50052}:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.docker.yaml
    - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_healthy
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
| ビルドコンテキスト | `./regions/system`（dockerfile: `./server/rust/config/Dockerfile`） |
| Dockerfile | マルチステージビルド（`rust:1.93-bookworm` → `debian:bookworm-slim`） |
| プロファイル | `system` |
| ポート | REST `${CONFIG_REST_HOST_PORT:-8084}:8080` / gRPC `${CONFIG_GRPC_HOST_PORT:-50054}:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（healthy） |
| 環境変数 | `CONFIG_PATH=/app/config/config.docker.yaml`, `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317`, `ALLOW_INSECURE_NO_AUTH=true` |
| ビルド引数 | `CARGO_FEATURES=k1s0-server-common/dev-auth-bypass`（認証バイパスコードをバイナリに含める） |
| ボリューム | `./regions/system/server/rust/config/config:/app/config` |

> **認証バイパスの仕組み**: `CARGO_FEATURES` ビルド引数で `k1s0-server-common/dev-auth-bypass` フィーチャーを有効化してビルドし、ランタイムの `ALLOW_INSECURE_NO_AUTH=true` と組み合わせることで認証バイパスが発動する。本番 Dockerfile ではビルド引数を指定しないため、バイパスコードはバイナリに含まれない。デバッグビルド（`cargo run`）では自動的にバイパス可能。

```yaml
config-rust:
  build:
    context: ./regions/system
    dockerfile: ./server/rust/config/Dockerfile
    args:
      # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
      CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  profiles: [system]
  ports:
    - "${CONFIG_REST_HOST_PORT:-8084}:8080"    # REST
    - "${CONFIG_GRPC_HOST_PORT:-50054}:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.docker.yaml
    - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
    # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
    - ALLOW_INSECURE_NO_AUTH=true
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_healthy
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
| ビルドコンテキスト | `./regions/system`（dockerfile: `./server/rust/saga/Dockerfile`） |
| Dockerfile | マルチステージビルド（`rust:1.93-bookworm` → `debian:bookworm-slim`） |
| プロファイル | `system` |
| ポート | REST `${SAGA_REST_HOST_PORT:-8085}:8080` / gRPC `${SAGA_GRPC_HOST_PORT:-50055}:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（healthy） |
| 環境変数 | `CONFIG_PATH=/app/config/config.docker.yaml`, `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317`, `ALLOW_INSECURE_NO_AUTH=true` |
| ビルド引数 | `CARGO_FEATURES=k1s0-server-common/dev-auth-bypass`（認証バイパスコードをバイナリに含める） |
| ボリューム | `./regions/system/server/rust/saga/config:/app/config` |

```yaml
saga-rust:
  build:
    context: ./regions/system
    dockerfile: ./server/rust/saga/Dockerfile
    args:
      # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
      CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  profiles: [system]
  ports:
    - "${SAGA_REST_HOST_PORT:-8085}:8080"    # REST
    - "${SAGA_GRPC_HOST_PORT:-50055}:50051"  # gRPC
  environment:
    - CONFIG_PATH=/app/config/config.docker.yaml
    - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
    # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
    - ALLOW_INSECURE_NO_AUTH=true
  depends_on:
    postgres:
      condition: service_healthy
    kafka:
      condition: service_healthy
    keycloak:
      condition: service_healthy
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
| ビルドコンテキスト | `./regions/system`（dockerfile: `./server/rust/dlq-manager/Dockerfile`） |
| Dockerfile | マルチステージビルド（`rust:1.93-bookworm` → `debian:bookworm-slim`） |
| プロファイル | `system` |
| ポート | REST `${DLQ_REST_HOST_PORT:-8086}:8080`（gRPC なし） |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy） |
| 環境変数 | `CONFIG_PATH=/app/config/config.docker.yaml`, `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317`, `ALLOW_INSECURE_NO_AUTH=true` |
| ビルド引数 | `CARGO_FEATURES=k1s0-server-common/dev-auth-bypass`（認証バイパスコードをバイナリに含める） |
| ボリューム | `./regions/system/server/rust/dlq-manager/config:/app/config` |

```yaml
dlq-manager:
  build:
    context: ./regions/system
    dockerfile: ./server/rust/dlq-manager/Dockerfile
    args:
      # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
      CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  profiles: [system]
  ports:
    - "${DLQ_REST_HOST_PORT:-8086}:8080"    # REST のみ（gRPC なし）
  environment:
    - CONFIG_PATH=/app/config/config.docker.yaml
    - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
    # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
    - ALLOW_INSECURE_NO_AUTH=true
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

全ホストポートは `${VAR:-default}` 形式で外部から変更可能。`.env` ファイルで設定する。

| サービス | REST ポート（デフォルト） | gRPC ポート（デフォルト） | 備考 |
| --- | --- | --- | --- |
| auth-rust | `${AUTH_REST_HOST_PORT:-8083}` | `${AUTH_GRPC_HOST_PORT:-50052}` | Rust 版 auth-server |
| config-rust | `${CONFIG_REST_HOST_PORT:-8084}` | `${CONFIG_GRPC_HOST_PORT:-50054}` | Rust 版 config-server |
| saga-rust | `${SAGA_REST_HOST_PORT:-8085}` | `${SAGA_GRPC_HOST_PORT:-50055}` | Rust 版 saga-server |
| dlq-manager | `${DLQ_REST_HOST_PORT:-8086}` | - | DLQ 管理サーバー（gRPC なし） |
| event-monitor-rust | `${EVENT_MONITOR_REST_HOST_PORT:-8095}` | `${EVENT_MONITOR_GRPC_HOST_PORT:-50200}` | ※50200 は Hyper-V 除外範囲回避（ADR-0040） |
| master-maintenance-rust | `${MASTER_MAINTENANCE_REST_HOST_PORT:-8098}` | `${MASTER_MAINTENANCE_GRPC_HOST_PORT:-50201}` | ※50201 は Hyper-V 除外範囲回避（ADR-0040） |
| navigation-rust | `${NAVIGATION_REST_HOST_PORT:-8099}` | `${NAVIGATION_GRPC_HOST_PORT:-50202}` | ※50202 は Hyper-V 除外範囲回避（ADR-0040） |
| policy-rust | `${POLICY_REST_HOST_PORT:-8101}` | `${POLICY_GRPC_HOST_PORT:-50203}` | ※50203 は Hyper-V 除外範囲回避（ADR-0040） |
| rule-engine-rust | `${RULE_ENGINE_REST_HOST_PORT:-8103}` | `${RULE_ENGINE_GRPC_HOST_PORT:-50204}` | ※50204 は Hyper-V 除外範囲回避（ADR-0040） |
| session-rust | `${SESSION_REST_HOST_PORT:-8106}` | `${SESSION_GRPC_HOST_PORT:-50205}` | ※50205 は Hyper-V 除外範囲回避（ADR-0040） |
| workflow-rust | `${WORKFLOW_REST_HOST_PORT:-8107}` | - | CRIT-2 監査対応: depends_on に scheduler-rust を追加 |

## アプリケーションサービスの追加

本ファイル（`docker-compose.yaml`）にはインフラサービスのみを定義する。アプリケーションサービスは `docker-compose.dev.yaml`（開発用オーバーライド）で管理し、開発時は `docker compose -f docker-compose.yaml -f docker-compose.dev.yaml up` で明示的に指定する。

### 方式

- リポジトリに `docker-compose.override.yaml.example` を配置し、テンプレートとして提供する
- 各開発者は `docker-compose.override.yaml.example` を参考にローカル設定を追加できる
- `docker-compose.override.yaml` は `.gitignore` で除外されており、自動読込による意図しないセキュリティ緩和を防止する
- 開発用オーバーライドは `docker-compose.dev.yaml` に定義し、`docker compose -f docker-compose.yaml -f docker-compose.dev.yaml up` で明示的に指定する

### docker-compose.override.yaml.example

各サービスの詳細設定は上記セクションを参照。

```yaml
# docker-compose.override.yaml.example
# このファイルを参考にして docker-compose.dev.yaml にサービス定義を追加してください。
# docker-compose.override.yaml は Docker Compose が自動読込するため使用しないでください。
#
# 使用方法: docker compose -f docker-compose.yaml -f docker-compose.dev.yaml up

services:
  # --- system tier ---
  # ポート割り当て（デフォルト値）:
  #   auth-rust:   REST 8083, gRPC 50052
  #   config-rust: REST 8084, gRPC 50054
  #   saga-rust:   REST 8085, gRPC 50055
  #   dlq-manager: REST 8086  (gRPC なし)
  # 全ポートは ${VAR:-default} 形式で .env から変更可能
  #
  # auth-rust:
  #   build:
  #     context: ./regions/system
  #     dockerfile: ./server/rust/auth/Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "${AUTH_REST_HOST_PORT:-8083}:8080"
  #     - "${AUTH_GRPC_HOST_PORT:-50052}:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.docker.yaml
  #     - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_healthy
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
  #     context: ./regions/system
  #     dockerfile: ./server/rust/config/Dockerfile
  #     args:
  #       # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
  #       CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  #   profiles: [system]
  #   ports:
  #     - "${CONFIG_REST_HOST_PORT:-8084}:8080"
  #     - "${CONFIG_GRPC_HOST_PORT:-50054}:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.docker.yaml
  #     - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
  #     # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
  #     - ALLOW_INSECURE_NO_AUTH=true
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_healthy
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
  #     context: ./regions/system
  #     dockerfile: ./server/rust/saga/Dockerfile
  #     args:
  #       # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
  #       CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  #   profiles: [system]
  #   ports:
  #     - "${SAGA_REST_HOST_PORT:-8085}:8080"
  #     - "${SAGA_GRPC_HOST_PORT:-50055}:50051"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.docker.yaml
  #     - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
  #     # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
  #     - ALLOW_INSECURE_NO_AUTH=true
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #     kafka:
  #       condition: service_healthy
  #     keycloak:
  #       condition: service_healthy
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
  #     context: ./regions/system
  #     dockerfile: ./server/rust/dlq-manager/Dockerfile
  #     args:
  #       # 開発環境用: 認証バイパスコードをバイナリに含める（本番では指定しない）
  #       CARGO_FEATURES: "k1s0-server-common/dev-auth-bypass"
  #   profiles: [system]
  #   ports:
  #     - "${DLQ_REST_HOST_PORT:-8086}:8080"
  #   environment:
  #     - CONFIG_PATH=/app/config/config.docker.yaml
  #     - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
  #     # CARGO_FEATURES でバイパスコードを含めたバイナリに対し、ランタイムで認証バイパスを有効化
  #     - ALLOW_INSECURE_NO_AUTH=true
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

  # --- service tier ---
  # task-server:
  #   build:
  #     context: ./regions/service/task/server/rust
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
  #     - ./regions/service/task/server/rust/config:/app/config

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

> **注記**: 本番環境では Kong は DB-backed モード（PostgreSQL）で運用し、decK で宣言的に管理する。詳細は [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md) を参照。

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
    # Status API はメトリクス・ヘルスチェックのみ提供（設定変更不可）
    # Docker 内部ネットワーク限定でホストへの ports 公開なし
    # Admin API（127.0.0.1:8001）とは異なり全 NIC バインドが安全
    KONG_STATUS_LISTEN: "0.0.0.0:8100"
    KONG_PROXY_ACCESS_LOG: /dev/stdout
    KONG_ADMIN_ACCESS_LOG: /dev/stdout
    KONG_PROXY_ERROR_LOG: /dev/stderr
    KONG_ADMIN_ERROR_LOG: /dev/stderr
    KONG_LOG_LEVEL: info
  ports:
    - "8000:8000"    # Proxy
    - "8001:8001"    # Admin API
    # Status API（8100）は ports に含めず Docker 内部ネットワークのみに限定する
  volumes:
    - ./infra/kong/kong.dev.yaml:/etc/kong/kong.yaml:ro
  healthcheck:
    test: ["CMD", "kong", "health"]
    interval: 10s
    timeout: 5s
    retries: 3
```

> **セキュリティ注記（Status API）**: `KONG_STATUS_LISTEN: "0.0.0.0:8100"` は全 NIC にバインドするが、`ports` セクションに 8100 を含めないことでホストへの公開を防止している。Status API はメトリクス取得（`/metrics`）とヘルスチェック（`/status`）のみを提供し、設定変更機能を持たないため、Docker 内部ネットワーク限定であれば全 NIC バインドは安全である。Prometheus は Docker ネットワーク内から直接 `kong:8100/metrics` にアクセスする。

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
FROM rust:1.93-bookworm AS builder

# 開発環境用の追加 Cargo フィーチャー（本番では空のまま）
ARG CARGO_FEATURES=""

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler cmake build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY . .
# CARGO_FEATURES が指定された場合、--features で追加フィーチャーを有効化してビルド
RUN touch src/main.rs && \
    if [ -n "$CARGO_FEATURES" ]; then \
      cargo build --release --features "$CARGO_FEATURES"; \
    else \
      cargo build --release; \
    fi

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/k1s0-auth-server /k1s0-auth-server
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/k1s0-auth-server"]
```

| 項目 | 設定 |
| --- | --- |
| ビルドイメージ | `rust:1.93-bookworm` |
| ランタイムイメージ | `debian:bookworm-slim`（C/C++ ランタイム含む） |
| 追加パッケージ | `protobuf-compiler`（tonic-build）, `cmake` + `build-essential`（rdkafka） |
| 依存キャッシュ | ダミー `main.rs` で依存クレートを先にビルド |
| ビルド引数 | `CARGO_FEATURES`（開発用フィーチャーの有効化、デフォルト空） |
| 実行ユーザー | `nonroot:nonroot` |

> **CARGO_FEATURES による認証バイパスの安全性設計**: docker-compose の `build.args` で `CARGO_FEATURES=k1s0-server-common/dev-auth-bypass` を指定すると、認証バイパスコードがバイナリにコンパイルされる。さらにランタイムの環境変数 `ALLOW_INSECURE_NO_AUTH=true` と組み合わせることで認証バイパスが発動する。本番 Dockerfile ではビルド引数を指定しないため、バイパスコードはバイナリに一切含まれず、環境変数だけでは認証をバイパスできない。ローカルのデバッグビルド（`cargo run`）では `cfg(debug_assertions)` により自動的にバイパス可能。

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
- [docker-compose-インフラサービス設計.md](compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis・Kong の詳細設定
- [docker-compose-可観測性サービス設計.md](compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定
- [system-server.md](../../servers/auth/server.md) -- auth-server 設計
- [system-config-server.md](../../servers/config/server.md) -- 設定管理サーバー設計
- [system-saga-server.md](../../servers/saga/server.md) -- Saga オーケストレーションサーバー設計
- [system-dlq-manager-server.md](../../servers/dlq-manager/server.md) -- DLQ 管理サーバー設計
- [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) -- Kong 構成管理
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
