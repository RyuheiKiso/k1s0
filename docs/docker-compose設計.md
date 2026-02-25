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
    image: confluentinc/cp-kafka:7.7.1
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

PostgreSQL の `docker-entrypoint-initdb.d` に配置し、Tier ごとのデータベースを自動作成する。データベースは認証用とアプリケーション用（Tier 別）に分離する。詳細なスキーマ定義は [docker-compose-インフラサービス設計.md](docker-compose-インフラサービス設計.md) の「PostgreSQL 初期化」セクションを参照。

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

## プロファイル組み合わせ表

### プロファイル一覧

| プロファイル | サービス | 用途 |
| --- | --- | --- |
| `infra` | PostgreSQL, MySQL, Redis, Redis-session, Kafka, Kafka-UI, Schema Registry, Keycloak, Vault, kafka-init | 共通インフラ |
| `observability` | Jaeger, Prometheus, Loki, Grafana | 監視・可視化 |
| `system` | auth-rust, config-rust | system 層サーバー |
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
docker compose --profile infra --profile system up -d --build auth-rust

# ログの確認
docker compose --profile infra --profile system logs -f auth-rust

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
| DB マイグレーション確認 | infra | `docker compose --profile infra up -d` |

## 設計上の補足

- ローカル開発用のパスワードは `dev` で統一する（本番シークレットとの混同を防ぐ）
- ヘルスチェックを全サービスに定義し、`depends_on` の `condition: service_healthy` で起動順序を制御する
- ボリューム名はサービス名に対応させ、`docker compose down -v` で一括削除できるようにする
- アプリケーションサーバーの設定ファイルはボリュームマウントで提供し、イメージの再ビルドなしに設定変更を反映できるようにする
- Kafka トピックの自動作成には `kafka-init` コンテナを使用し、ブローカー起動後に一度だけ実行する
- Kong はローカル開発環境では DB-less モード（declarative config）を使用し、本番環境との差異を最小限にしつつ開発効率を優先する

## 詳細設計ドキュメント

各サービスの詳細設定は以下の分割ドキュメントを参照。

- [docker-compose-システムサービス設計.md](docker-compose-システムサービス設計.md) -- auth-server・config-server・System プロファイルの詳細設定・Kong ローカル設定
- [docker-compose-インフラサービス設計.md](docker-compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis の詳細設定・初期化スクリプト
- [docker-compose-可観測性サービス設計.md](docker-compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定

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
