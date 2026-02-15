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

PostgreSQL の `docker-entrypoint-initdb.d` に配置し、Tier ごとのデータベースを自動作成する。データベースは認証用とアプリケーション用（Tier 別）に分離する。

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

```yaml
# docker-compose.override.yaml.example
# このファイルを docker-compose.override.yaml にコピーして使用してください。
# 必要なサービスのコメントを解除して起動してください。

services:
  # --- system 層 ---
  # auth-server:
  #   build:
  #     context: ./regions/system/server/go/auth
  #     dockerfile: Dockerfile
  #   profiles: [system]
  #   ports:
  #     - "8083:8080"    # 8081 は schema-registry、8082 は order-server が使用するため 8083 を割り当て
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
  #   volumes:
  #     - ./regions/system/server/go/auth/config:/app/config

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
  # kong:
  #   image: kong:3.7
  #   profiles: [infra]
  #   environment:
  #     KONG_DATABASE: postgres
  #     KONG_PG_HOST: postgres
  #     KONG_PG_DATABASE: kong
  #     KONG_PG_USER: dev
  #     KONG_PG_PASSWORD: dev
  #     KONG_PROXY_ACCESS_LOG: /dev/stdout
  #     KONG_ADMIN_ACCESS_LOG: /dev/stdout
  #     KONG_PROXY_ERROR_LOG: /dev/stderr
  #     KONG_ADMIN_ERROR_LOG: /dev/stderr
  #     KONG_ADMIN_LISTEN: "0.0.0.0:8001"
  #   ports:
  #     - "8000:8000"
  #     - "8001:8001"
  #   depends_on:
  #     postgres:
  #       condition: service_healthy
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

## 設計上の補足

- ローカル開発用のパスワードは `dev` で統一する（本番シークレットとの混同を防ぐ）
- ヘルスチェックを全サービスに定義し、`depends_on` の `condition: service_healthy` で起動順序を制御する
- ボリューム名はサービス名に対応させ、`docker compose down -v` で一括削除できるようにする

## 関連ドキュメント

- [config設計](config設計.md)
- [devcontainer設計](devcontainer設計.md)
- [インフラ設計](インフラ設計.md)
- [可観測性設計](可観測性設計.md)
- [メッセージング設計](メッセージング設計.md)
