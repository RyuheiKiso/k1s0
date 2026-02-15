# docker-compose 設計

ローカル開発環境で使用する `docker-compose.yaml` の設計を定義する。

## 基本方針

- 開発者が必要なサービスだけを起動できるよう、Compose プロファイルで分類する
- 依存インフラ（DB・Kafka・Redis 等）は共通プロファイルで提供する
- アプリケーションサービスは階層・種別ごとにプロファイルを割り当てる
- ボリュームでデータを永続化し、コンテナ再作成時もデータを保持する

## プロファイル設計

| プロファイル  | 対象                                     |
| ------------- | ---------------------------------------- |
| infra         | PostgreSQL, MySQL, Redis, Kafka 等       |
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
    ports:
      - "8090:8080"
    depends_on:
      kafka:
        condition: service_healthy

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
      - "3200:3000"
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
  kafka-data:
  prometheus-data:
  loki-data:
  grafana-data:

networks:
  default:
    name: k1s0-network
```

## DB 初期化スクリプト

PostgreSQL の `docker-entrypoint-initdb.d` に配置し、サービスごとのデータベースを自動作成する。

```sql
-- infra/docker/init-db/01-create-databases.sql
CREATE DATABASE auth_db;
CREATE DATABASE order_db;
-- 必要に応じて追加
```

## アプリケーションサービスの追加

CLI の「ひな形生成」で生成された各サービスは、個別の `docker-compose.override.yaml` または本ファイルへの追記で登録する。

```yaml
# アプリケーションサービスの例
services:
  auth-server:
    build:
      context: ./regions/system/server/go/auth
      dockerfile: Dockerfile
    profiles: [system]
    ports:
      - "8081:8080"
    depends_on:
      postgres:
        condition: service_healthy
    volumes:
      - ./regions/system/server/go/auth/config:/app/config

  order-server:
    build:
      context: ./regions/service/order/server/go
      dockerfile: Dockerfile
    profiles: [service]
    ports:
      - "8082:8080"
    depends_on:
      postgres:
        condition: service_healthy
      kafka:
        condition: service_healthy
    volumes:
      - ./regions/service/order/server/go/config:/app/config
```

## 設計上の補足

- ローカル開発用のパスワードは `dev` で統一する（本番シークレットとの混同を防ぐ）
- ヘルスチェックを全サービスに定義し、`depends_on` の `condition: service_healthy` で起動順序を制御する
- ボリューム名はサービス名に対応させ、`docker compose down -v` で一括削除できるようにする
