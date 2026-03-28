# docker-compose 設計

ローカル開発環境で使用する `docker-compose.yaml` の設計を定義する。

## 基本方針

- 開発者が必要なサービスだけを起動できるよう、Compose プロファイルで分類する
- 依存インフラ（DB・Kafka・Redis 等）は共通プロファイルで提供する
- アプリケーションサービスは `docker-compose.dev.yaml`（開発用オーバーライド）で管理し、本ファイルにはインフラサービスのみ定義する
- ボリュームでデータを永続化し、コンテナ再作成時もデータを保持する
- **RDBMS 方針**: PostgreSQL を標準 RDBMS とする。MySQL は既存システム連携用として残す。SQL Server は当プロジェクトでは採用しない

## プロファイル依存関係図

![Docker Composeプロファイル依存関係図](images/profile-dependency.svg)

## プロファイル設計

| プロファイル  | 対象                                     |
| ------------- | ---------------------------------------- |
| infra         | PostgreSQL, MySQL, Redis, Kafka, Keycloak 等 |
| observability | Jaeger, Prometheus, Grafana, Loki        |
| system        | system tier のサーバー・DB                  |
| business      | business tier のサーバー・クライアント・DB  |
| service       | service tier のサーバー・クライアント・DB   |

### プロファイル依存チェーン（HIGH-01 対応）

プロファイル間には暗黙の依存関係がある。各プロファイルのサービスは下位プロファイルのサービスに `depends_on` を持つため、**依存先のプロファイルを同時に指定しないとコンテナが起動しない**。

```
infra → system → business → service
```

| 起動したいプロファイル | 必要な --profile 指定 |
| --- | --- |
| infra のみ | `--profile infra` |
| system | `--profile infra --profile system` |
| business | `--profile infra --profile system --profile business` |
| service | `--profile infra --profile system --profile business --profile service` |

> **注意**: `--profile system` のみを指定して起動すると、`depends_on` で参照する `postgres`・`kafka`・`keycloak`（infra プロファイル）が存在せずエラーになる。必ず依存先のプロファイルを含めて起動すること。

**推奨**: `just local-up` を使用する。`justfile` が `--profile infra --profile system` を自動的に付与する（`_dc_profiles` 変数）。

### 使用例

```bash
# 推奨: just local-up を使用（--profile infra --profile system を自動適用）
just local-up

# インフラのみ起動（DB・Redis・Kafka）
docker compose --profile infra up -d

# インフラ + 可観測性
docker compose --profile infra --profile observability up -d

# infra + system（手動起動例）
docker compose --profile infra --profile system up -d

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
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: postgres:17.4
    profiles: [infra]
    restart: unless-stopped
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-dev}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-dev}
    ports:
      - "${PG_HOST_PORT:-5432}:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./infra/docker/init-db:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-dev}"]
      interval: 5s
      timeout: 3s
      retries: 5

  mysql:
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: mysql:8.4.4
    profiles: [infra]
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD:-dev}
      MYSQL_USER: ${MYSQL_USER:-dev}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD:-dev}
    ports:
      - "${MYSQL_HOST_PORT:-3306}:3306"
    volumes:
      - mysql-data:/var/lib/mysql
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: redis:7.4.2
    profiles: [infra]
    restart: unless-stopped
    ports:
      - "${REDIS_HOST_PORT:-6379}:6379"
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
    image: apache/kafka:3.8.0
    profiles: [infra]
    environment:
      KAFKA_NODE_ID: 1
      KAFKA_PROCESS_ROLES: broker,controller
      KAFKA_CONTROLLER_QUORUM_VOTERS: 1@kafka:9093
      KAFKA_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
      CLUSTER_ID: "5L6g3nShT-eMCtK--X86sw"
      KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS: 0
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 1
      KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: 1
    ports:
      # ホストのループバックインターフェースのみにバインド（MED-5 監査対応: 外部ネットワークからの不正アクセスを防止）
      - "127.0.0.1:${KAFKA_HOST_PORT:-9092}:9092"
    volumes:
      - kafka-data:/var/lib/kafka
    healthcheck:
      test: ["CMD-SHELL", "bash -lc 'kafka-broker-api-versions.sh --bootstrap-server localhost:9092'"]
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
      - "${KAFKA_UI_HOST_PORT:-8090}:8080"
    depends_on:
      kafka:
        condition: service_healthy
      schema-registry:
        condition: service_healthy

  schema-registry:
    image: confluentinc/cp-schema-registry:7.7.1
    profiles: [infra]
    environment:
      SCHEMA_REGISTRY_HOST_NAME: schema-registry
      SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS: kafka:9092
      SCHEMA_REGISTRY_LISTENERS: http://0.0.0.0:8081
      SCHEMA_REGISTRY_SCHEMA_REGISTRY_LEADER_CONNECT_TIMEOUT_MS: 120000
    ports:
      - "${SCHEMA_REGISTRY_HOST_PORT:-8081}:8081"
    depends_on:
      kafka:
        condition: service_healthy
    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:8081/ || exit 1"]
      interval: 15s
      timeout: 5s
      retries: 10
      start_period: 30s

  keycloak:
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: quay.io/keycloak/keycloak:26.0.7
    profiles: [infra]
    environment:
      KC_DB: postgres
      KC_DB_URL_HOST: postgres
      KC_DB_URL_DATABASE: keycloak
      KC_DB_USERNAME: ${KC_DB_USERNAME:-dev}
      KC_DB_PASSWORD: ${KC_DB_PASSWORD:-dev}
      # Keycloak 26以降は KC_BOOTSTRAP_ADMIN_* を使用する（KEYCLOAK_ADMIN_* は非推奨）
      KC_BOOTSTRAP_ADMIN_USERNAME: ${KEYCLOAK_ADMIN:-admin}
      KC_BOOTSTRAP_ADMIN_PASSWORD: ${KEYCLOAK_ADMIN_PASSWORD:-dev}
      KC_HEALTH_ENABLED: "true"
    entrypoint: ["/bin/bash", "/opt/keycloak/data/import/docker-entrypoint-wrapper.sh"]
    command: ["start-dev", "--import-realm"]
    ports:
      - "${KEYCLOAK_HOST_PORT:-8180}:8080"
      - "${KEYCLOAK_MGMT_HOST_PORT:-9000}:9000"
    volumes:
      - ./infra/docker/keycloak:/opt/keycloak/data/import    # realm k1s0 の初期設定。config.dev.yaml の auth.jwt.issuer（realms/k1s0）と一致させること
    depends_on:
      postgres:
        condition: service_healthy

  redis-session:
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: redis:7.4.2
    profiles: [infra]
    ports:
      # ホストのループバックインターフェースのみにバインド（MED-4 監査対応: 外部ネットワークからの不正アクセスを防止）
      - "127.0.0.1:${REDIS_SESSION_HOST_PORT:-6380}:6379"
    volumes:
      - redis-session-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  vault:
    # パッチバージョン固定: 意図しない自動アップグレードを防止する（L-07）
    image: hashicorp/vault:1.17.6
    profiles: [infra]
    cap_add:
      - IPC_LOCK
    environment:
      VAULT_DEV_ROOT_TOKEN_ID: ${VAULT_DEV_ROOT_TOKEN_ID:-dev-token}
    ports:
      - "${VAULT_HOST_PORT:-8200}:8200"

  # ============================================================
  # 可観測性
  # NOTE: ローカル開発環境では observability プロファイル有効時のみ Promtail を起動する
  #       （`docker compose --profile observability up`）。
  # Kubernetes 環境では Promtail を DaemonSet としてデプロイする。
  # ============================================================
  jaeger:
    image: jaegertracing/all-in-one:1.62
    profiles: [observability]
    environment:
      COLLECTOR_OTLP_ENABLED: "true"
    ports:
      - "${JAEGER_UI_HOST_PORT:-16686}:16686"   # UI
      - "${JAEGER_OTLP_GRPC_HOST_PORT:-4317}:4317"     # OTLP gRPC
      - "${JAEGER_OTLP_HTTP_HOST_PORT:-4318}:4318"     # OTLP HTTP

  prometheus:
    image: prom/prometheus:v2.55
    profiles: [observability]
    volumes:
      - ./infra/docker/prometheus/prometheus.yaml:/etc/prometheus/prometheus.yml
      - ./infra/docker/prometheus/recording_rules.yaml:/etc/prometheus/recording_rules.yaml
      - ./infra/docker/prometheus/alerting_rules.yaml:/etc/prometheus/alerting_rules.yaml
      - prometheus-data:/prometheus
    ports:
      - "${PROMETHEUS_HOST_PORT:-9090}:9090"

  loki:
    image: grafana/loki:3.3
    profiles: [observability]
    command: -config.file=/etc/loki/loki-config.yaml
    ports:
      - "${LOKI_HOST_PORT:-3100}:3100"
    volumes:
      - ./infra/docker/loki/loki-config.yaml:/etc/loki/loki-config.yaml:ro
      - loki-data:/loki

  grafana:
    image: grafana/grafana:11.3
    profiles: [observability]
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GF_SECURITY_ADMIN_PASSWORD:-dev}
    ports:
      - "${GRAFANA_HOST_PORT:-3200}:3000"   # ホストポート 3200 を使用（3000 はフロントエンド開発サーバー等とのポート競合を回避するため）
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
    name: ${COMPOSE_NETWORK_NAME:-k1s0-network}
```

## Kafka 接続設定（config.yaml 例）

ローカル開発（docker-compose）では Kafka は `PLAINTEXT` を使用する。一方で staging/prod（Kubernetes）では `SASL_SSL` を使用するため、アプリケーションの Kafka 接続設定は **必ず config.yaml で環境ごとに切り替える**。

### dev（docker-compose / PLAINTEXT）

```yaml
kafka:
  brokers:
    - "kafka:9092"
  security_protocol: "PLAINTEXT"
  consumer_group: "{service-name}.dev"
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
      - "k1s0.system.auth.login.v1"
    subscribe:
      - "k1s0.system.config.changed.v1"
      - "k1s0.system.auth.login.v1"
```

### staging/prod（Kubernetes / SASL_SSL）

```yaml
kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9093"
    - "kafka-1.messaging.svc.cluster.local:9093"
    - "kafka-2.messaging.svc.cluster.local:9093"
  security_protocol: "SASL_SSL"
  consumer_group: "{service-name}.default"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: "${KAFKA_SASL_USERNAME}"  # Vault 等から注入
    password: "${KAFKA_SASL_PASSWORD}"  # Vault 等から注入
  tls:
    ca_cert_path: "/etc/kafka/certs/ca.crt"  # Strimzi が発行する CA 証明書
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
      - "k1s0.system.auth.login.v1"
    subscribe:
      - "k1s0.system.config.changed.v1"
      - "k1s0.system.auth.login.v1"
```

詳細なフィールド定義・命名規則は [config設計](../../cli/config/config設計.md) と [メッセージング設計](../../architecture/messaging/メッセージング設計.md) を参照。

## Vault 初期化スクリプト

ローカル開発用の Vault 初期化スクリプト（`infra/docker/vault/init-vault.sh`）が実装済みである。`docker compose --profile infra up -d` で Vault が起動した後に手動実行する。

```bash
# Vault 起動後に実行（プロジェクトルートから）
bash infra/docker/vault/init-vault.sh
```

スクリプトは以下を自動設定する:

| 設定内容 | シークレットパス |
| --- | --- |
| Database 共通設定 | `secret/k1s0/system/database` |
| Auth Server DB / API キー | `secret/k1s0/system/auth-server/*` |
| Config Server DB / API キー | `secret/k1s0/system/config-server/*` |
| Saga Server DB | `secret/k1s0/system/saga-server/database` |
| DLQ Manager DB | `secret/k1s0/system/dlq-manager/database` |
| Kafka SASL | `secret/k1s0/system/kafka/sasl` |
| Keycloak クライアントシークレット | `secret/k1s0/system/keycloak` |

> **注記**: Vault はローカル開発環境では dev モード（`VAULT_DEV_ROOT_TOKEN_ID: ${VAULT_DEV_ROOT_TOKEN_ID:-dev-token}`）で起動する。KV v2 シークレットエンジンは `secret/` パスにデフォルトで有効化されている。

## DB 初期化スクリプト

PostgreSQL の `docker-entrypoint-initdb.d` に配置し、Tier ごとのデータベースを自動作成する。データベースは認証用とアプリケーション用（Tier 別）に分離する。詳細なスキーマ定義は [docker-compose-インフラサービス設計.md](compose-インフラサービス設計.md) の「PostgreSQL 初期化」セクションを参照。

初期化スクリプトは `infra/docker/init-db/` 配下の9ファイルで構成される。

| ファイル | 内容 |
| --- | --- |
| `01-create-databases.sql` | 全12データベース作成 |
| `02-auth-schema.sql` | auth-server スキーマ |
| `03-config-schema.sql` | config-server スキーマ |
| `04-saga-schema.sql` | saga-server スキーマ |
| `05-dlq-schema.sql` | dlq-manager スキーマ |
| `06-featureflag-schema.sql` | featureflag-server スキーマ |
| `07-ratelimit-schema.sql` | ratelimit-server スキーマ |
| `08-tenant-schema.sql` | tenant-server スキーマ |
| `09-vault-schema.sql` | vault スキーマ |

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

-- サービス個別DB
CREATE DATABASE auth_db;
CREATE DATABASE config_db;
CREATE DATABASE dlq_db;
CREATE DATABASE featureflag_db;
CREATE DATABASE ratelimit_db;
CREATE DATABASE tenant_db;
CREATE DATABASE vault_db;
```

## プロファイル組み合わせ表

### プロファイル一覧

| プロファイル | サービス | 用途 |
| --- | --- | --- |
| `infra` | PostgreSQL, MySQL, Redis, Redis-session, Kafka, Kafka-UI, Schema Registry, Keycloak, Vault, kafka-init | 共通インフラ |
| `observability` | Jaeger, Prometheus, Loki, Grafana | 監視・可視化 |
| `system` | auth-rust, config-rust, saga-rust, dlq-manager, featureflag-rust, ratelimit-rust, tenant-rust, vault-rust, graphql-gateway-rust, bff-proxy, api-registry-rust, app-registry-rust, event-monitor-rust, event-store-rust, file-rust, master-maintenance-rust, navigation-rust, notification-rust, policy-rust, quota-rust, rule-engine-rust, scheduler-rust, search-rust, service-catalog-rust, session-rust, workflow-rust, ai-gateway-rust, ai-agent-rust | system tier サーバー |
| `business` | project-master-rust | business tier サーバー |
| `service` | task-rust, board-rust, activity-rust | service tier サーバー |

### 起動コマンド一覧

```bash
# インフラのみ起動（DB・Redis・Kafka・Keycloak）
docker compose --profile infra up -d

# インフラ + 可観測性
docker compose --profile infra --profile observability up -d

# インフラ + system tier サーバー
docker compose --profile infra --profile system up -d

# インフラ + 可観測性 + system tier
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

- ローカル開発用のクレデンシャルはすべて `${VAR:-default}` パターンで環境変数化する。デフォルト値は `dev` で統一し、`.env` で上書き可能とする（本番シークレットとの混同を防ぎつつ、共用サーバー等での個別設定にも対応）
- ヘルスチェックを全サービスに定義し、`depends_on` の `condition: service_healthy` で起動順序を制御する
- ボリューム名はサービス名に対応させ、`docker compose down -v` で一括削除できるようにする
- アプリケーションサーバーの設定ファイルはボリュームマウントで提供し、イメージの再ビルドなしに設定変更を反映できるようにする
- Kafka トピックの自動作成には `kafka-init` コンテナを使用し、ブローカー起動後に一度だけ実行する。`restart: on-failure` を設定し、Kafka ブローカーが一時的に未準備の場合にリトライする
- Kong はローカル開発環境では DB-less モード（declarative config）を使用し、本番環境との差異を最小限にしつつ開発効率を優先する。設定ファイルは `./infra/kong/kong.dev.yaml` をマウントする（`/etc/kong/kong.yaml` にマップ）
- Kong ローカル開発用の設定には以下の4プラグインを使用する: `cors`（開発元オリジン許可）、`rate-limiting`（グローバル 5000 req/min、local policy）、`jwt`（Keycloak JWKS 連携、RS256、有効期限 900s）、`prometheus`（per_consumer メトリクス収集）
- distroless / scratch ベースのコンテナ（bff-proxy 等）では `curl` が利用できないため、ビルド時に busybox をコピーし、`/busybox wget` でヘルスチェックを行う。YAML アンカー `x-rust-healthcheck` で Rust サーバー共通のヘルスチェック定義を再利用する
- `vault-rust` の Vault 依存条件は `condition: service_healthy` とし、Vault のヘルスチェック通過を保証する（`service_started` ではヘルスチェック未通過で接続失敗の可能性がある）

### 再起動ポリシー（restart policy）

技術監査対応として、全インフラサービスに **`restart: unless-stopped`** を追加した。これにより、コンテナがクラッシュした場合に Docker が自動的に再起動し、開発者の手動介入なしにサービスが復旧する。明示的に `docker compose stop` / `docker compose down` で停止した場合は再起動しない。

| ポリシー | 適用対象 | 動作 |
|---------|---------|------|
| `unless-stopped` | postgres, mysql, redis, redis-session, kafka, kafka-ui, schema-registry, keycloak, vault 等のインフラサービス | クラッシュ時に自動再起動。明示停止時は再起動しない |
| `on-failure` | kafka-init（初期化コンテナ） | 失敗時のみリトライ。成功後は再起動しない |

各サービスにはリソース制限（`deploy.resources.limits`）も併せて設定し、コンテナがホストのリソースを過剰に消費することを防止する。

## 共用開発サーバー対応

ローカル PC にリソース集約型サービスを起動する容量がない場合、共用開発サーバーを活用できる。VS Code Remote SSH + `COMPOSE_PROJECT_NAME` による Compose Project 分離を推奨方式とする。

詳細は [共用開発サーバー設計](../devenv/共用開発サーバー設計.md) を参照。

## .env 戦略

| ファイル | git 管理 | 用途 |
| --- | --- | --- |
| `.env.example` | 対象 | 環境変数テンプレート。デフォルト値のリファレンス |
| `.env` | 除外 | 開発者個人の設定。`.env.example` をコピーして使用 |
| `docker-compose.dev.yaml` | 管理対象 | 開発用オーバーライド（認証バイパス等。明示的に `-f` 指定が必要） system/business/service 全プロファイルのサービスを網羅すること |
| `docker-compose.override.yaml` | 除外 | Docker Compose 自動読込防止のため使用禁止 |

`.env` が存在しない場合、`docker-compose.yaml` 内の `${VAR:-default}` 記法によりデフォルト値が適用される。そのため、既存のローカル開発ワークフローには影響しない（後方互換）。

```bash
# 初回セットアップ
cp .env.example .env

# 共用サーバーの場合はユーザー名を設定
echo "COMPOSE_PROJECT_NAME=$(whoami)" >> .env
```

## COMPOSE_PROJECT_NAME による分離

`COMPOSE_PROJECT_NAME` を設定すると、Docker Compose は以下のリソースに自動的にプレフィックスを付与する。

| リソース | デフォルト | COMPOSE_PROJECT_NAME=alice |
| --- | --- | --- |
| コンテナ名 | `k1s0-postgres-1` | `alice-postgres-1` |
| ボリューム名 | `k1s0_postgres-data` | `alice_postgres-data` |
| ネットワーク名 | `k1s0-network` | `alice_default` |

これにより、共用サーバー上で複数開発者が同時に `docker compose up` しても、リソースが完全に分離される。

## クレデンシャル環境変数一覧

全クレデンシャルは `${VAR:-default}` 形式で環境変数化されており、`.env` で上書き可能。未設定時はデフォルト値（`dev` 等）が適用される。

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `POSTGRES_USER` | dev | postgres | PostgreSQL ユーザー名 |
| `POSTGRES_PASSWORD` | dev | postgres | PostgreSQL パスワード |
| `MYSQL_ROOT_PASSWORD` | dev | mysql | MySQL root パスワード |
| `MYSQL_USER` | dev | mysql | MySQL ユーザー名 |
| `MYSQL_PASSWORD` | dev | mysql | MySQL パスワード |
| `KC_DB_USERNAME` | dev | keycloak | Keycloak DB ユーザー名 |
| `KC_DB_PASSWORD` | dev | keycloak | Keycloak DB パスワード |
| `KEYCLOAK_ADMIN` | admin | keycloak | Keycloak 管理者ユーザー名（`KC_BOOTSTRAP_ADMIN_USERNAME` にマップ） |
| `KEYCLOAK_ADMIN_PASSWORD` | dev | keycloak | Keycloak 管理者パスワード（`KC_BOOTSTRAP_ADMIN_PASSWORD` にマップ） |
| `VAULT_DEV_ROOT_TOKEN_ID` | dev-token | vault | Vault dev モードトークン |
| `GF_SECURITY_ADMIN_PASSWORD` | dev | grafana | Grafana 管理者パスワード |

## ポート環境変数一覧

全ホストポートは `${VAR:-default}` 形式で環境変数化されており、`.env` で変更可能。

### インフラサービス

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `PG_HOST_PORT` | 5432 | postgres | PostgreSQL |
| `MYSQL_HOST_PORT` | 3306 | mysql | MySQL |
| `REDIS_HOST_PORT` | 6379 | redis | Redis |
| `REDIS_SESSION_HOST_PORT` | 6380 | redis-session | Redis（BFF セッション用） |
| `KAFKA_HOST_PORT` | 9092 | kafka | Kafka ブローカー |
| `KAFKA_UI_HOST_PORT` | 8090 | kafka-ui | Kafka UI |
| `SCHEMA_REGISTRY_HOST_PORT` | 8081 | schema-registry | Schema Registry |
| `KEYCLOAK_HOST_PORT` | 8180 | keycloak | Keycloak |
| `KEYCLOAK_MGMT_HOST_PORT` | 9000 | keycloak | Keycloak 管理（ヘルスチェック） |
| `VAULT_HOST_PORT` | 8200 | vault | HashiCorp Vault |

### API Gateway

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `KONG_PROXY_HOST_PORT` | 8000 | kong | Kong Proxy |
| `KONG_ADMIN_HOST_PORT` | 8001 | kong | Kong Admin API |

### System サービス

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `BFF_PROXY_HOST_PORT` | 8082 | bff-proxy | BFF Proxy（REST） |
| `AUTH_REST_HOST_PORT` | 8083 | auth-rust | Auth Server（REST） |
| `AUTH_GRPC_HOST_PORT` | 50052 | auth-rust | Auth Server（gRPC） |
| `CONFIG_REST_HOST_PORT` | 8084 | config-rust | Config Server（REST） |
| `CONFIG_GRPC_HOST_PORT` | 50054 | config-rust | Config Server（gRPC） |
| `SAGA_REST_HOST_PORT` | 8085 | saga-rust | Saga Server（REST） |
| `SAGA_GRPC_HOST_PORT` | 50055 | saga-rust | Saga Server（gRPC） |
| `DLQ_REST_HOST_PORT` | 8086 | dlq-manager | DLQ Manager（REST） |
| `FEATUREFLAG_REST_HOST_PORT` | 8087 | featureflag-rust | Feature Flag（REST） |
| `FEATUREFLAG_GRPC_HOST_PORT` | 50056 | featureflag-rust | Feature Flag（gRPC） |
| `RATELIMIT_REST_HOST_PORT` | 8088 | ratelimit-rust | Rate Limit（REST） |
| `RATELIMIT_GRPC_HOST_PORT` | 50057 | ratelimit-rust | Rate Limit（gRPC） |
| `TENANT_REST_HOST_PORT` | 8089 | tenant-rust | Tenant Server（REST） |
| `TENANT_GRPC_HOST_PORT` | 50058 | tenant-rust | Tenant Server（gRPC） |
| `VAULT_SVC_REST_HOST_PORT` | 8091 | vault-rust | Vault Service（REST） |
| `VAULT_SVC_GRPC_HOST_PORT` | 50059 | vault-rust | Vault Service（gRPC） |
| `GRAPHQL_GW_HOST_PORT` | 8092 | graphql-gateway-rust | GraphQL Gateway |
| `API_REGISTRY_REST_HOST_PORT` | 8093 | api-registry-rust | API Registry（REST） |
| `APP_REGISTRY_REST_HOST_PORT` | 8094 | app-registry-rust | App Registry（REST） |
| `EVENT_MONITOR_REST_HOST_PORT` | 8095 | event-monitor-rust | Event Monitor（REST） |
| `EVENT_MONITOR_GRPC_HOST_PORT` | 50200 | event-monitor-rust | Event Monitor（gRPC） |
| `EVENT_STORE_REST_HOST_PORT` | 8096 | event-store-rust | Event Store（REST） |
| `FILE_REST_HOST_PORT` | 8097 | file-rust | File Service（REST） |
| `MASTER_MAINTENANCE_REST_HOST_PORT` | 8098 | master-maintenance-rust | Master Maintenance（REST） |
| `MASTER_MAINTENANCE_GRPC_HOST_PORT` | 50201 | master-maintenance-rust | Master Maintenance（gRPC） |
| `NAVIGATION_REST_HOST_PORT` | 8099 | navigation-rust | Navigation（REST） |
| `NAVIGATION_GRPC_HOST_PORT` | 50202 | navigation-rust | Navigation（gRPC） |
| `NOTIFICATION_REST_HOST_PORT` | 8100 | notification-rust | Notification（REST） |
| `POLICY_REST_HOST_PORT` | 8101 | policy-rust | Policy（REST） |
| `POLICY_GRPC_HOST_PORT` | 50203 | policy-rust | Policy（gRPC） |
| `QUOTA_REST_HOST_PORT` | 8102 | quota-rust | Quota（REST） |
| `RULE_ENGINE_REST_HOST_PORT` | 8103 | rule-engine-rust | Rule Engine（REST） |
| `RULE_ENGINE_GRPC_HOST_PORT` | 50204 | rule-engine-rust | Rule Engine（gRPC） |
| `SCHEDULER_REST_HOST_PORT` | 8104 | scheduler-rust | Scheduler（REST） |
| `SEARCH_REST_HOST_PORT` | 8105 | search-rust | Search（REST） |
| `SESSION_REST_HOST_PORT` | 8106 | session-rust | Session（REST） |
| `SESSION_GRPC_HOST_PORT` | 50205 | session-rust | Session（gRPC） |
| `WORKFLOW_REST_HOST_PORT` | 8107 | workflow-rust | Workflow（REST） |
| `SERVICE_CATALOG_REST_HOST_PORT` | 8108 | service-catalog-rust | Service Catalog（REST） |
| `AI_GATEWAY_REST_HOST_PORT` | 8120 | ai-gateway-rust | AI Gateway（REST） |
| `AI_GATEWAY_GRPC_HOST_PORT` | 50071 | ai-gateway-rust | AI Gateway（gRPC） |
| `AI_AGENT_REST_HOST_PORT` | 8121 | ai-agent-rust | AI Agent（REST） |
| `AI_AGENT_GRPC_HOST_PORT` | 50072 | ai-agent-rust | AI Agent（gRPC） |

### Business サービス

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `PROJECT_MASTER_REST_HOST_PORT` | 8210 | project-master-rust | Project Master（REST） |
| `PROJECT_MASTER_GRPC_HOST_PORT` | 9210 | project-master-rust | Project Master（gRPC） |

### Service サービス

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `TASK_REST_HOST_PORT` | 8310 | task-rust | Task（REST） |
| `TASK_GRPC_HOST_PORT` | 9310 | task-rust | Task（gRPC） |
| `BOARD_REST_HOST_PORT` | 8320 | board-rust | Board（REST） |
| `BOARD_GRPC_HOST_PORT` | 9320 | board-rust | Board（gRPC） |
| `ACTIVITY_REST_HOST_PORT` | 8330 | activity-rust | Activity（REST） |
| `ACTIVITY_GRPC_HOST_PORT` | 9330 | activity-rust | Activity（gRPC） |

### 可観測性サービス

| 環境変数 | デフォルト | サービス | 説明 |
| --- | --- | --- | --- |
| `JAEGER_UI_HOST_PORT` | 16686 | jaeger | Jaeger UI |
| `JAEGER_OTLP_GRPC_HOST_PORT` | 4317 | jaeger | OTLP gRPC |
| `JAEGER_OTLP_HTTP_HOST_PORT` | 4318 | jaeger | OTLP HTTP |
| `PROMETHEUS_HOST_PORT` | 9090 | prometheus | Prometheus |
| `LOKI_HOST_PORT` | 3100 | loki | Loki |
| `GRAFANA_HOST_PORT` | 3200 | grafana | Grafana |

### ネットワーク

| 環境変数 | デフォルト | 説明 |
| --- | --- | --- |
| `COMPOSE_NETWORK_NAME` | k1s0-network | Docker ネットワーク名 |

## プロファイル依存関係と起動順序

### プロファイル詳細

Docker Compose で定義されている全プロファイルと、それぞれに含まれるサービス・依存関係を以下に示す。

#### `infra` プロファイル

共通インフラサービス。他の全プロファイルの前提となる。

| サービス | イメージ | 依存先 | 用途 |
|---------|---------|--------|------|
| postgres | postgres:17.4 | なし | PostgreSQL（主要 RDBMS） |
| mysql | mysql:8.4.4 | なし | MySQL（既存システム連携用） |
| redis | redis:7.4.2 | なし | Redis（キャッシュ・分散ロック） |
| redis-session | redis:7.4.2 | なし | Redis（BFF セッション管理用） |
| kafka | apache/kafka:3.8.0 | なし | Kafka メッセージブローカー |
| kafka-ui | provectuslabs/kafka-ui:v0.7.2 | kafka, schema-registry | Kafka 管理 UI |
| schema-registry | confluentinc/cp-schema-registry:7.7.1 | kafka | Avro/JSON Schema レジストリ |
| keycloak | quay.io/keycloak/keycloak:26.0.7 | postgres | 認証・認可基盤（IdP） |
| vault | hashicorp/vault:1.17.6 | なし | シークレット管理 |
| kong | kong:3.9.1 | なし | API Gateway |
| kafka-init | apache/kafka:3.8.0 | kafka | Kafka トピック自動作成（初期化タスク） |

#### `observability` プロファイル

可観測性スタック。`infra` プロファイルとは独立して起動可能。

| サービス | イメージ | 依存先 | 用途 |
|---------|---------|--------|------|
| jaeger | jaegertracing/all-in-one:1.62 | なし | 分散トレーシング |
| prometheus | prom/prometheus:v2.55 | なし | メトリクス収集・アラート |
| loki | grafana/loki:3.3 | なし | ログ集約 |
| promtail | grafana/promtail:2.9.0 | loki | ログ転送（Docker ログ収集） |
| grafana | grafana/grafana:11.3 | prometheus, loki, jaeger | ダッシュボード・可視化 |

#### `system` プロファイル

System Tier のアプリケーションサーバー群。`infra` プロファイルに依存する。`observability` は任意。

| サービス | 依存先（infra） | 用途 |
|---------|----------------|------|
| auth-rust | postgres, kafka, keycloak | 認証サーバー |
| config-rust | postgres, kafka, keycloak | 設定管理サーバー |
| saga-rust | postgres, kafka, keycloak | Saga オーケストレーター |
| dlq-manager | postgres, kafka | Dead Letter Queue 管理 |
| featureflag-rust | postgres | フィーチャーフラグ |
| ratelimit-rust | postgres, redis | レート制限 |
| tenant-rust | postgres, keycloak | テナント管理 |
| vault-rust | postgres, vault | Vault サービスラッパー |
| api-registry-rust | postgres | API レジストリ |
| app-registry-rust | postgres, kafka | アプリケーションレジストリ |
| event-monitor-rust | postgres, kafka | イベント監視 |
| event-store-rust | postgres | イベントストア |
| file-rust | postgres | ファイル管理 |
| master-maintenance-rust | postgres | マスタメンテナンス |
| navigation-rust | postgres | ナビゲーション |
| notification-rust | postgres, kafka | 通知 |
| policy-rust | postgres | ポリシー管理 |
| quota-rust | postgres, redis | クォータ管理 |
| rule-engine-rust | postgres | ルールエンジン |
| scheduler-rust | postgres, kafka | スケジューラー |
| search-rust | postgres | 検索 |
| service-catalog-rust | postgres | サービスカタログ |
| session-rust | postgres, redis | セッション管理 |
| workflow-rust | postgres, kafka, **scheduler-rust** | ワークフロー（CRIT-2 監査対応: 起動時に scheduler-rust へ接続するため依存を追加） |
| ai-gateway-rust | postgres | AI Gateway |
| ai-agent-rust | postgres, ai-gateway-rust | AI Agent |
| graphql-gateway-rust | auth-rust, tenant-rust 他多数 | GraphQL 統合ゲートウェイ |
| bff-proxy | keycloak, redis-session | BFF プロキシ（Go） |

#### `business` プロファイル

Business Tier のアプリケーションサーバー群。`infra` + `system` プロファイルに依存する。

| サービス | 依存先 | 用途 |
|---------|--------|------|
| project-master-rust | postgres, kafka, auth-rust | プロジェクトマスタ管理 |

#### `service` プロファイル

Service Tier のアプリケーションサーバー群。`infra` プロファイルに依存する。

| サービス | 依存先 | 用途 |
|---------|--------|------|
| task-rust | postgres, kafka | タスク管理 |
| board-rust | postgres, kafka | ボード管理 |
| activity-rust | postgres, kafka | アクティビティ管理 |

> **注意**: service/business プロファイルのサービスは `docker-compose.dev.yaml` で `dev-auth-bypass` と `DATABASE_URL` のオーバーライドが必要。k1s0 ユーザーのパスワードは `init-db/00-create-app-user.sh` のデフォルト値（`dev-k1s0-local`）と一致させること。新しいサービスを追加した場合は `docker-compose.dev.yaml` にも記載を追加すること。

### 推奨起動順序

プロファイル間の依存関係に基づく推奨起動順序を以下に示す。

```
1. infra          — 全サービスの前提（DB・キャッシュ・メッセージブローカー・認証基盤）
2. observability  — 任意。メトリクス・トレース・ログの収集基盤
3. system         — infra に依存。System Tier の全サーバー
4. business       — infra + system に依存。Business Tier のサーバー
5. service        — infra に依存。Service Tier のサーバー
```

> **注意**: `business` プロファイルの `project-master-rust` は `auth-rust`（system プロファイル）に依存するため、`system` プロファイルを先に起動する必要がある。`graphql-gateway-rust` は多数の system サービスに `depends_on` を持つため、最後に起動される。

### 全プロファイル起動コマンド

```bash
# 推奨順序に従った全プロファイル起動
docker compose \
  --profile infra \
  --profile observability \
  --profile system \
  --profile business \
  --profile service \
  up -d

# 開発用（認証バイパス有効）
docker compose \
  -f docker-compose.yaml \
  -f docker-compose.dev.yaml \
  --profile infra \
  --profile observability \
  --profile system \
  --profile business \
  --profile service \
  up -d
```

## 詳細設計ドキュメント

各サービスの詳細設定は以下の分割ドキュメントを参照。

- [docker-compose-システムサービス設計.md](compose-システムサービス設計.md) -- auth-server・config-server・System プロファイルの詳細設定・Kong ローカル設定
- [docker-compose-インフラサービス設計.md](compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis の詳細設定・初期化スクリプト
- [docker-compose-可観測性サービス設計.md](compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定
- [.env.example](../../../.env.example) -- ポート環境変数テンプレート

## 関連ドキュメント

- [config設計](../../cli/config/config設計.md)
- [devcontainer設計](../devenv/devcontainer設計.md)
- [インフラ設計](../overview/インフラ設計.md)
- [可観測性設計](../../architecture/observability/可観測性設計.md)
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md)
- [ディレクトリ構成図](../../architecture/overview/ディレクトリ構成図.md)
- [system-server設計](../../servers/auth/server.md)
- [system-config-server設計](../../servers/config/server.md)
- [認証認可設計](../../architecture/auth/認証認可設計.md)
- [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md)
- [Dockerイメージ戦略](Dockerイメージ戦略.md)
- [テンプレート仕様-DockerCompose](../../templates/infrastructure/DockerCompose.md)
- [共用開発サーバー設計](../devenv/共用開発サーバー設計.md)
