# docker-compose 設計

ローカル開発環境で使用する `docker-compose.yaml` の設計を定義する。

## 基本方針

- 開発者が必要なサービスだけを起動できるよう、Compose プロファイルで分類する
- 依存インフラ（DB・Kafka・Redis 等）は共通プロファイルで提供する
- アプリケーションサービスは `docker-compose.override.yaml` で管理し、本ファイルにはインフラサービスのみ定義する
- ボリュームでデータを永続化し、コンテナ再作成時もデータを保持する
- **RDBMS 方針**: PostgreSQL を標準 RDBMS とする。MySQL は既存システム連携用として残す。SQL Server は当プロジェクトでは採用しない

## プロファイル設計

| プロファイル  | 対象                                     |
| ------------- | ---------------------------------------- |
| infra         | PostgreSQL, MySQL, Redis, Kafka, Keycloak 等 |
| observability | Jaeger, Prometheus, Grafana, Loki        |
| system        | system 層のサーバー・DB                  |
| business      | business 層のサーバー・クライアント・DB  |
| service       | service 層のサーバー・クライアント・DB   |

### 使用例

```bash
# インフラのみ起動（DB・Redis・Kafka）
docker compose --profile infra up -d

# インフラ + 可観測性
docker compose --profile infra --profile observability up -d

# 全サービス起動
docker compose --profile infra --profile observability --profile system --profile business --profile service up -d
```

## docker-compose.yaml

```yaml
# docker-compose.yaml

services:
  # ============================================================
  # インフラ
  # ============================================================
  postgres:
    image: postgres:17
    profiles: [infra]
    environment:
      POSTGRES_USER: dev
      POSTGRES_PASSWORD: dev
    ports:
      - "5432:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./infra/docker/init-db:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U dev"]
      interval: 5s
      timeout: 3s
      retries: 5

  mysql:
    image: mysql:8.4
    profiles: [infra]
    environment:
      MYSQL_ROOT_PASSWORD: dev
      MYSQL_USER: dev
      MYSQL_PASSWORD: dev
    ports:
      - "3306:3306"
    volumes:
      - mysql-data:/var/lib/mysql
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7
    profiles: [infra]
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  # NOTE: ローカル開発では PLAINTEXT を使用（開発効率優先）。
  # staging/prod では SASL_SSL を使用し、Strimzi Operator が証明書管理を行う。
  # ローカルと staging/prod でセキュリティプロトコルが異なるため、
  # 接続設定は必ず config.yaml 経由で環境ごとに切り替えること。
  kafka:
    image: bitnami/kafka:3.8
    profiles: [infra]
    environment:
      KAFKA_CFG_NODE_ID: 0
      KAFKA_CFG_PROCESS_ROLES: broker,controller
      KAFKA_CFG_CONTROLLER_QUORUM_VOTERS: 0@kafka:9093
      KAFKA_CFG_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
      KAFKA_CFG_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_CFG_CONTROLLER_LISTENER_NAMES: CONTROLLER
      KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
    ports:
      - "9092:9092"
    volumes:
      - kafka-data:/bitnami/kafka
    healthcheck:
      test: ["CMD-SHELL", "kafka-broker-api-versions.sh --bootstrap-server localhost:9092"]
      interval: 10s
      timeout: 5s
      retries: 5

  kafka-ui:
    image: provectuslabs/kafka-ui:latest
    profiles: [infra]
    environment:
      KAFKA_CLUSTERS_0_NAME: local
      KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:9092
      KAFKA_CLUSTERS_0_SCHEMAREGISTRY: http://schema-registry:8081
    ports:
      - "8090:8080"
    depends_on:
      kafka:
        condition: service_healthy
      schema-registry:
        condition: service_healthy

  schema-registry:
    image: confluentinc/cp-schema-registry:7.7
    profiles: [infra]
    environment:
      SCHEMA_REGISTRY_HOST_NAME: schema-registry
      SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS: kafka:9092
      SCHEMA_REGISTRY_LISTENERS: http://0.0.0.0:8081
    ports:
      - "8081:8081"
    depends_on:
      kafka:
        condition: service_healthy
    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:8081/ || exit 1"]
      interval: 10s
      timeout: 5s
      retries: 5

  keycloak:
    image: quay.io/keycloak/keycloak:26.0
    profiles: [infra]
    environment:
      KC_DB: postgres
      KC_DB_URL_HOST: postgres
      KC_DB_URL_DATABASE: keycloak
      KC_DB_USERNAME: dev
      KC_DB_PASSWORD: dev
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: dev
    command: start-dev --import-realm
    ports:
      - "8180:8080"
    volumes:
      - ./infra/docker/keycloak:/opt/keycloak/data/import    # realm k1s0 の初期設定。config.dev.yaml の auth.jwt.issuer（realms/k1s0）と一致させること
    depends_on:
      postgres:
        condition: service_healthy

  redis-session:
    image: redis:7
    profiles: [infra]
    ports:
      - "6380:6379"
    volumes:
      - redis-session-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  vault:
    image: hashicorp/vault:1.17
    profiles: [infra]
    cap_add:
      - IPC_LOCK
    environment:
      VAULT_DEV_ROOT_TOKEN_ID: dev-token
    ports:
      - "8200:8200"

  # ============================================================
  # 可観測性
  # NOTE: ローカル開発環境では Promtail を省略している。
  # Kubernetes 環境では Promtail（DaemonSet）がログを収集し Loki に転送するが、
  # ローカルでは各コンテナの stdout を直接 docker compose logs で確認する。
  # ============================================================
  jaeger:
    image: jaegertracing/all-in-one:1.62
    profiles: [observability]
    environment:
      COLLECTOR_OTLP_ENABLED: "true"
    ports:
      - "16686:16686"   # UI
      - "4317:4317"     # OTLP gRPC
      - "4318:4318"     # OTLP HTTP

  prometheus:
    image: prom/prometheus:v2.55
    profiles: [observability]
    volumes:
      - ./infra/docker/prometheus/prometheus.yaml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"

  loki:
    image: grafana/loki:3.3
    profiles: [observability]
    ports:
      - "3100:3100"
    volumes:
      - loki-data:/loki

  grafana:
    image: grafana/grafana:11.3
    profiles: [observability]
    environment:
      GF_SECURITY_ADMIN_PASSWORD: dev
    ports:
      - "3200:3000"   # ホストポート 3200 を使用（3000 はフロントエンド開発サーバー等とのポート競合を回避するため）
    volumes:
      - grafana-data:/var/lib/grafana
      - ./infra/docker/grafana/provisioning:/etc/grafana/provisioning
      - ./infra/docker/grafana/dashboards:/var/lib/grafana/dashboards
    depends_on:
      - prometheus
      - loki
      - jaeger

volumes:
  postgres-data:
  mysql-data:
  redis-data:
  redis-session-data:
  kafka-data:
  prometheus-data:
  loki-data:
  grafana-data:

networks:
  default:
    name: k1s0-network
```

## DB 初期化スクリプト

PostgreSQL の `docker-entrypoint-initdb.d` に配置し、Tier ごとのデータベースを自動作成する。データベースは認証用とアプリケーション用（Tier 別）に分離する。詳細なスキーマ定義は「インフラサービス詳細設定 > PostgreSQL 初期化」セクションを参照。

```sql
-- infra/docker/init-db/01-create-databases.sql

-- 認証用DB（Keycloak）
CREATE DATABASE keycloak;

-- API ゲートウェイ用DB（Kong）
CREATE DATABASE kong;

-- アプリケーション用DB（Tier ごとに分離）
CREATE DATABASE k1s0_system;
CREATE DATABASE k1s0_business;
CREATE DATABASE k1s0_service;
```

## アプリケーションサービスの追加

本ファイル（`docker-compose.yaml`）にはインフラサービスのみを定義する。アプリケーションサービスは `docker-compose.override.yaml` で管理し、各開発者がローカルで必要なサービスのみを起動できるようにする。

### 方式

- リポジトリに `docker-compose.override.yaml.example` を配置し、テンプレートとして提供する
- 各開発者は `docker-compose.override.yaml.example` をコピーして `docker-compose.override.yaml` を作成する
- `docker-compose.override.yaml` は `.gitignore` に追加し、各開発者のローカル設定として管理する
- Docker Compose は `docker-compose.yaml` と `docker-compose.override.yaml` を自動的にマージする

### docker-compose.override.yaml.example

各サービスの詳細設定は「System プロファイル サービス定義」セクションを参照。

```yaml
# docker-compose.override.yaml.example
# このファイルを docker-compose.override.yaml にコピーして使用してください。
# 必要なサービスのコメントを解除して起動してください。

services:
  # --- system 層 ---
  # ポート割り当て:
  #   auth-go:    REST 8080, gRPC 50051
  #   auth-rust:  REST 8083, gRPC 50052
  #   config-go:  REST 8082, gRPC 50053
  #   config-rust: REST 8084, gRPC 50054
  #
  # auth-go:
  #   build:
  #     context: ./regions/system/server/go/auth
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8080:8080"
  #     - "50051:50051"
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
  #     - ./regions/system/server/go/auth/config:/app/config
  #   healthcheck:
  #     test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
  #     interval: 10s
  #     timeout: 5s
  #     retries: 5
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
  # config-go:
  #   build:
  #     context: ./regions/system/server/go/config
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8082:8080"
  #     - "50053:50051"
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
  #     - ./regions/system/server/go/config/config:/app/config
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

  # --- service 層 ---
  # order-server:
  #   build:
  #     context: ./regions/service/order/server/go
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
  #     - ./regions/service/order/server/go/config:/app/config

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

## System プロファイル サービス定義

system 層のアプリケーションサーバーは `docker-compose.override.yaml` で管理する。以下に各サービスの詳細設定を示す。

### auth-server（Go 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `auth-go` |
| ビルドコンテキスト | `./regions/system/server/go/auth` |
| Dockerfile | マルチステージビルド（`golang:1.23-bookworm` → `gcr.io/distroless/static-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8080:8080` / gRPC `50051:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（started） |
| 環境変数 | `CONFIG_PATH=/app/config/config.dev.yaml` |
| ボリューム | `./regions/system/server/go/auth/config:/app/config` |

```yaml
auth-go:
  build:
    context: ./regions/system/server/go/auth
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8080:8080"    # REST
    - "50051:50051"  # gRPC
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
    - ./regions/system/server/go/auth/config:/app/config
  healthcheck:
    test: ["CMD-SHELL", "curl -f http://localhost:8080/healthz || exit 1"]
    interval: 10s
    timeout: 5s
    retries: 5
```

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

### config-server（Go 版）

| 項目 | 設定 |
| --- | --- |
| サービス名 | `config-go` |
| ビルドコンテキスト | `./regions/system/server/go/config` |
| Dockerfile | マルチステージビルド（`golang:1.23-bookworm` → `gcr.io/distroless/static-debian12:nonroot`） |
| プロファイル | `system` |
| ポート | REST `8082:8080` / gRPC `50053:50051` |
| 依存サービス | `postgres`（healthy）, `kafka`（healthy）, `keycloak`（started） |
| 環境変数 | `CONFIG_PATH=/app/config/config.dev.yaml` |
| ボリューム | `./regions/system/server/go/config/config:/app/config` |

```yaml
config-go:
  build:
    context: ./regions/system/server/go/config
    dockerfile: Dockerfile
  profiles: [system]
  ports:
    - "8082:8080"    # REST
    - "50053:50051"  # gRPC
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
    - ./regions/system/server/go/config/config:/app/config
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

### ポート割り当て一覧

| サービス | REST ポート | gRPC ポート | 備考 |
| --- | --- | --- | --- |
| auth-go | 8080 | 50051 | Go 版 auth-server |
| config-go | 8082 | 50053 | Go 版 config-server |
| auth-rust | 8083 | 50052 | Rust 版 auth-server |
| config-rust | 8084 | 50054 | Rust 版 config-server |

> **注記**: Go 版と Rust 版は同一機能の並行実装であり、通常は一方のみを起動する。両方を同時に起動する場合はポートが競合しないよう上記の通りホストポートを分離している。

## インフラサービス詳細設定

### PostgreSQL 初期化

PostgreSQL の `docker-entrypoint-initdb.d` でデータベースとスキーマを自動初期化する。

#### データベース作成

```sql
-- infra/docker/init-db/01-create-databases.sql

-- 認証用DB（Keycloak）
CREATE DATABASE keycloak;

-- API ゲートウェイ用DB（Kong）
CREATE DATABASE kong;

-- アプリケーション用DB（Tier ごとに分離）
CREATE DATABASE k1s0_system;
CREATE DATABASE k1s0_business;
CREATE DATABASE k1s0_service;
```

#### auth-server 用スキーマ

```sql
-- infra/docker/init-db/02-auth-schema.sql

\c k1s0_system;

-- 監査ログテーブル
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    user_id VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    resource VARCHAR(500),
    action VARCHAR(10),
    result VARCHAR(20) NOT NULL,
    metadata JSONB DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_recorded_at ON audit_logs(recorded_at);
```

#### config-server 用スキーマ

```sql
-- infra/docker/init-db/03-config-schema.sql

\c k1s0_system;

-- 設定値テーブル
CREATE TABLE IF NOT EXISTS config_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace VARCHAR(255) NOT NULL,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT DEFAULT '',
    created_by VARCHAR(255) NOT NULL,
    updated_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(namespace, key)
);

CREATE INDEX idx_config_entries_namespace ON config_entries(namespace);

-- 設定変更監査ログ
CREATE TABLE IF NOT EXISTS config_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_id UUID NOT NULL REFERENCES config_entries(id),
    namespace VARCHAR(255) NOT NULL,
    key VARCHAR(255) NOT NULL,
    old_value JSONB,
    new_value JSONB NOT NULL,
    old_version INTEGER,
    new_version INTEGER NOT NULL,
    changed_by VARCHAR(255) NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_config_audit_namespace ON config_audit_logs(namespace);
CREATE INDEX idx_config_audit_changed_at ON config_audit_logs(changed_at);
```

### Keycloak 初期設定

Keycloak は `start-dev --import-realm` で起動し、realm 設定を自動インポートする。

| 項目 | 設定 |
| --- | --- |
| Realm 名 | `k1s0` |
| Admin ユーザー | `admin` / `dev` |
| DB | PostgreSQL（`keycloak` データベース） |
| インポートパス | `./infra/docker/keycloak/` |
| ポート | `8180:8080` |

#### Realm エクスポートファイル

```json
// infra/docker/keycloak/k1s0-realm.json（主要部分）
{
  "realm": "k1s0",
  "enabled": true,
  "sslRequired": "none",
  "roles": {
    "realm": [
      { "name": "user", "description": "一般ユーザー" },
      { "name": "sys_auditor", "description": "監査担当" },
      { "name": "sys_operator", "description": "運用担当" },
      { "name": "sys_admin", "description": "システム管理者" }
    ]
  },
  "clients": [
    {
      "clientId": "react-spa",
      "publicClient": true,
      "redirectUris": ["http://localhost:3000/*"],
      "webOrigins": ["http://localhost:3000"],
      "standardFlowEnabled": true,
      "directAccessGrantsEnabled": false,
      "attributes": {
        "pkce.code.challenge.method": "S256"
      }
    },
    {
      "clientId": "k1s0-cli",
      "publicClient": true,
      "standardFlowEnabled": false,
      "directAccessGrantsEnabled": false,
      "attributes": {
        "oauth2.device.authorization.grant.enabled": "true"
      }
    },
    {
      "clientId": "k1s0-bff",
      "publicClient": false,
      "secret": "dev-bff-secret",
      "serviceAccountsEnabled": true,
      "standardFlowEnabled": true,
      "redirectUris": ["http://localhost:8080/callback"],
      "webOrigins": ["http://localhost:8080"]
    }
  ],
  "users": [
    {
      "username": "dev-admin",
      "email": "dev-admin@example.com",
      "enabled": true,
      "credentials": [{ "type": "password", "value": "dev" }],
      "realmRoles": ["user", "sys_admin"]
    },
    {
      "username": "dev-user",
      "email": "dev-user@example.com",
      "enabled": true,
      "credentials": [{ "type": "password", "value": "dev" }],
      "realmRoles": ["user"]
    }
  ]
}
```

### Kafka トピック自動作成

ローカル開発環境では、`kafka-init` コンテナでトピックを自動作成する。

```yaml
kafka-init:
  image: bitnami/kafka:3.8
  profiles: [infra]
  depends_on:
    kafka:
      condition: service_healthy
  entrypoint: ["/bin/bash", "-c"]
  command:
    - |
      echo "Creating Kafka topics..."
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.auth.audit.v1 --partitions 6 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.config.changed.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.service.order.created.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.service.order.updated.v1 --partitions 3 --replication-factor 1
      echo "Kafka topics created."
  restart: "no"
```

> **注記**: トピック命名規則は `k1s0.{tier}.{domain}.{event-type}.{version}` に従う。詳細は [メッセージング設計](メッセージング設計.md) を参照。

### Redis

| サービス | 用途 | ポート | ボリューム |
| --- | --- | --- | --- |
| `redis` | キャッシュ / レート制限 | 6379 | `redis-data` |
| `redis-session` | BFF セッションストア | 6380 | `redis-session-data` |

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
    url: http://auth-go:8080
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
    url: http://config-go:8080
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
    url: http://auth-go:8080
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

> **注記**: ローカル開発環境では JWT プラグインは無効にしている（開発効率を優先）。E2E テストで JWT 検証を含めたい場合は、`kong.dev.yaml` に `jwt` プラグインを追加すること。

## Observability サービス詳細設定

可観測性サービスの詳細設定を定義する。設計の全体像は [可観測性設計](可観測性設計.md) を参照。

### Prometheus scrape 設定

```yaml
# infra/docker/prometheus/prometheus.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: "auth-server"
    static_configs:
      - targets: ["auth-go:8080", "auth-rust:8080"]
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: "config-server"
    static_configs:
      - targets: ["config-go:8080", "config-rust:8080"]
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: "kong"
    static_configs:
      - targets: ["kong:8001"]
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: "kafka"
    static_configs:
      - targets: ["kafka:9092"]
    scrape_interval: 30s
```

### Grafana 自動プロビジョニング

#### データソース

```yaml
# infra/docker/grafana/provisioning/datasources/datasources.yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    editable: false

  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
    editable: false
```

#### ダッシュボードプロビジョニング

```yaml
# infra/docker/grafana/provisioning/dashboards/dashboards.yaml
apiVersion: 1

providers:
  - name: "k1s0"
    orgId: 1
    folder: "k1s0"
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: false
```

### Loki 設定

```yaml
# ローカル開発用（シングルインスタンス）
# Kubernetes 環境では Promtail（DaemonSet）がログを収集し Loki に転送するが、
# ローカルでは各コンテナの stdout を直接 docker compose logs で確認する。
# Loki はダッシュボード経由でのログ検索用途で提供する。
```

### Jaeger 設定

```yaml
# OTLP プロトコルで各サービスからトレースデータを受信する
# - OTLP gRPC: 4317（サービスからの送信先）
# - OTLP HTTP: 4318（HTTP 経由の送信先）
# - UI: 16686（Jaeger UI）
```

| 項目 | 設定 |
| --- | --- |
| OTLP gRPC | `jaeger:4317` |
| OTLP HTTP | `jaeger:4318` |
| UI | `localhost:16686` |

## マルチステージビルド Dockerfile 設計

各サーバーの Dockerfile はマルチステージビルドで最小限のランタイムイメージを生成する。詳細は [Dockerイメージ戦略](Dockerイメージ戦略.md) を参照。

### Go サーバー

```dockerfile
# regions/system/server/go/auth/Dockerfile
# Build stage
FROM golang:1.23-bookworm AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /auth ./cmd/server

# Runtime stage
FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /auth /auth
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/auth"]
```

| 項目 | 設定 |
| --- | --- |
| ビルドイメージ | `golang:1.23-bookworm` |
| ランタイムイメージ | `gcr.io/distroless/static-debian12:nonroot` |
| CGO | 無効（`CGO_ENABLED=0`） |
| バイナリ最適化 | `-ldflags="-s -w"`（デバッグ情報・シンボル除去） |
| 実行ユーザー | `nonroot:nonroot` |

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

> **注記**: distroless イメージには `curl` が含まれないため、ビルドステージで `curl` バイナリをコピーするか、Go の場合は自前のヘルスチェックバイナリを使用する。ローカル開発環境では Docker Compose の `healthcheck` ではなく `depends_on` の条件で起動順序を制御するため、distroless 内の curl 有無は問題にならない。

## 初期化スクリプト設計

### ディレクトリ構成

```
infra/docker/
├── init-db/
│   ├── 01-create-databases.sql    # データベース作成
│   ├── 02-auth-schema.sql         # auth-server 用スキーマ
│   └── 03-config-schema.sql       # config-server 用スキーマ
├── keycloak/
│   └── k1s0-realm.json            # Keycloak realm 初期設定
├── prometheus/
│   └── prometheus.yaml            # Prometheus scrape 設定
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── datasources.yaml   # データソース自動設定
│   │   └── dashboards/
│   │       └── dashboards.yaml    # ダッシュボードプロビジョニング
│   └── dashboards/
│       └── (JSON ダッシュボードファイル)
└── kong/
    ├── kong.yaml                  # Kong 本番用 decK 設定
    └── kong.dev.yaml              # Kong ローカル開発用 declarative config
```

### 初期化順序

1. **PostgreSQL 起動** → `docker-entrypoint-initdb.d` の SQL が番号順に実行される
2. **Keycloak 起動** → PostgreSQL の `keycloak` DB に接続し、realm 設定をインポート
3. **Kafka 起動** → `kafka-init` コンテナがトピックを自動作成
4. **アプリケーションサーバー起動** → `depends_on` + `condition: service_healthy` で依存サービスの準備完了を待機

### PostgreSQL 初期化の仕組み

PostgreSQL の公式 Docker イメージは、初回起動時に `/docker-entrypoint-initdb.d/` 配下のファイルをファイル名の辞書順で実行する。

| ファイル | 実行順 | 内容 |
| --- | --- | --- |
| `01-create-databases.sql` | 1 | データベース作成（keycloak, kong, k1s0_system, k1s0_business, k1s0_service） |
| `02-auth-schema.sql` | 2 | 監査ログテーブル作成（k1s0_system DB） |
| `03-config-schema.sql` | 3 | 設定値テーブル・設定変更監査ログテーブル作成（k1s0_system DB） |

> **注記**: 初期化スクリプトはデータボリュームが空の場合のみ実行される。既存データがある場合はスキップされるため、スキーマ変更時は `docker compose down -v` でボリュームを削除してから再起動すること。

### Keycloak Realm プロビジョニング

Keycloak は `start-dev --import-realm` オプションで起動し、`/opt/keycloak/data/import/` にマウントされた JSON ファイルから realm 設定を自動インポートする。

| 項目 | 設定 |
| --- | --- |
| Realm | `k1s0` |
| クライアント | `react-spa`（SPA用 PKCE）, `k1s0-cli`（CLI用 Device Auth）, `k1s0-bff`（BFF用） |
| ロール | `user`, `sys_auditor`, `sys_operator`, `sys_admin` |
| テストユーザー | `dev-admin`（sys_admin）, `dev-user`（user） |

### Kafka トピック自動作成

`kafka-init` コンテナが Kafka ブローカーのヘルスチェック完了後に、必要なトピックを作成する。

| トピック | パーティション数 | 用途 |
| --- | --- | --- |
| `k1s0.system.auth.audit.v1` | 6 | 認証監査ログ |
| `k1s0.system.config.changed.v1` | 3 | 設定変更通知 |
| `k1s0.service.order.created.v1` | 3 | 注文作成イベント |
| `k1s0.service.order.updated.v1` | 3 | 注文更新イベント |

## プロファイル組み合わせ表

### プロファイル一覧

| プロファイル | サービス | 用途 |
| --- | --- | --- |
| `infra` | PostgreSQL, MySQL, Redis, Redis-session, Kafka, Kafka-UI, Schema Registry, Keycloak, Vault, kafka-init | 共通インフラ |
| `observability` | Jaeger, Prometheus, Loki, Grafana | 監視・可視化 |
| `system` | auth-go/auth-rust, config-go/config-rust | system 層サーバー |
| `business` | (将来追加) | business 層サーバー |
| `service` | order-server (将来追加) | service 層サーバー |

### 起動コマンド一覧

```bash
# インフラのみ起動（DB・Redis・Kafka・Keycloak）
docker compose --profile infra up -d

# インフラ + 可観測性
docker compose --profile infra --profile observability up -d

# インフラ + system 層サーバー
docker compose --profile infra --profile system up -d

# インフラ + 可観測性 + system 層
docker compose --profile infra --profile observability --profile system up -d

# インフラ + 可観測性 + 全アプリケーション
docker compose --profile infra --profile observability --profile system --profile business --profile service up -d

# 特定サービスのみ再ビルドして起動
docker compose --profile infra --profile system up -d --build auth-go

# ログの確認
docker compose --profile infra --profile system logs -f auth-go

# 全サービス停止（データ保持）
docker compose --profile infra --profile observability --profile system down

# 全サービス停止 + データ削除（クリーンスタート）
docker compose --profile infra --profile observability --profile system down -v
```

### 開発シナリオ別の推奨プロファイル

| シナリオ | プロファイル | コマンド |
| --- | --- | --- |
| auth-server の開発 | infra + system | `docker compose --profile infra --profile system up -d` |
| config-server の開発 | infra + system | `docker compose --profile infra --profile system up -d` |
| フロントエンド開発（API モック不要） | infra + system | `docker compose --profile infra --profile system up -d` |
| パフォーマンス測定 | infra + observability + system | `docker compose --profile infra --profile observability --profile system up -d` |
| E2E テスト実行 | infra + observability + system + service | `docker compose --profile infra --profile observability --profile system --profile service up -d` |
| DB マイグレーション確認 | infra | `docker compose --profile infra up -d` |

## 設計上の補足

- ローカル開発用のパスワードは `dev` で統一する（本番シークレットとの混同を防ぐ）
- ヘルスチェックを全サービスに定義し、`depends_on` の `condition: service_healthy` で起動順序を制御する
- ボリューム名はサービス名に対応させ、`docker compose down -v` で一括削除できるようにする
- アプリケーションサーバーの設定ファイルはボリュームマウントで提供し、イメージの再ビルドなしに設定変更を反映できるようにする
- Kafka トピックの自動作成には `kafka-init` コンテナを使用し、ブローカー起動後に一度だけ実行する
- Kong はローカル開発環境では DB-less モード（declarative config）を使用し、本番環境との差異を最小限にしつつ開発効率を優先する

## 関連ドキュメント

- [config設計](config設計.md)
- [devcontainer設計](devcontainer設計.md)
- [インフラ設計](インフラ設計.md)
- [可観測性設計](可観測性設計.md)
- [メッセージング設計](メッセージング設計.md)
- [ディレクトリ構成図](ディレクトリ構成図.md)
- [system-server設計](system-server設計.md)
- [system-config-server設計](system-config-server設計.md)
- [認証認可設計](認証認可設計.md)
- [APIゲートウェイ設計](APIゲートウェイ設計.md)
- [Dockerイメージ戦略](Dockerイメージ戦略.md)
- [テンプレート仕様-DockerCompose](テンプレート仕様-DockerCompose.md)
