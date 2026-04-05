# docker-compose インフラサービス設計

docker-compose における PostgreSQL・Keycloak・Kafka・Redis・Kong の詳細設定を定義する。基本方針・プロファイル設計は [docker-compose設計.md](docker-compose設計.md) を参照。

---

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

-- auth-server 用DB
CREATE DATABASE auth_db;

-- config-server 用DB
CREATE DATABASE config_db;

-- dlq-manager 用DB
CREATE DATABASE dlq_db;

-- featureflag-server 用DB
CREATE DATABASE featureflag_db;

-- ratelimit-server 用DB
CREATE DATABASE ratelimit_db;

-- tenant-server 用DB
CREATE DATABASE tenant_db;

-- vault-server 用DB
CREATE DATABASE vault_db;
```

#### auth-server 用スキーマ

```sql
-- infra/docker/init-db/02-auth-schema.sql

\c k1s0_system;

-- 監査ログテーブル（auth スキーマ。詳細は system-database.md 参照）
-- ローカル開発では sqlx-cli のマイグレーションで auth.audit_logs が作成される
-- 以下は参照用の簡略版スキーマ
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    event_type VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(255),
    resource_id VARCHAR(255),
    result VARCHAR(50) NOT NULL DEFAULT 'SUCCESS',
    detail JSONB,
    ip_address INET,
    user_agent TEXT,
    trace_id VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
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

#### saga-server 用スキーマ

```sql
-- infra/docker/init-db/04-saga-schema.sql

\connect k1s0_system;

CREATE SCHEMA IF NOT EXISTS saga;

-- saga_states: Saga ワークフローの状態管理
CREATE TABLE saga.saga_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name VARCHAR(255) NOT NULL,
    current_step INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'STARTED',
    payload JSONB,
    correlation_id VARCHAR(255),
    initiated_by VARCHAR(255),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- saga_step_logs: 各ステップの実行ログ
CREATE TABLE saga.saga_step_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id UUID NOT NULL REFERENCES saga.saga_states(id),
    step_index INT NOT NULL,
    step_name VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    request_payload JSONB,
    response_payload JSONB,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

#### dlq-manager 用スキーマ

```sql
-- infra/docker/init-db/05-dlq-schema.sql

\connect dlq_db;

CREATE SCHEMA IF NOT EXISTS dlq;

-- dlq_messages: Dead Letter Queue メッセージ管理
CREATE TABLE IF NOT EXISTS dlq.dlq_messages (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    original_topic  VARCHAR(255) NOT NULL,
    error_message   TEXT         NOT NULL,
    retry_count     INT          NOT NULL DEFAULT 0,
    max_retries     INT          NOT NULL DEFAULT 3,
    payload         JSONB,
    status          VARCHAR(50)  NOT NULL DEFAULT 'PENDING',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_retry_at   TIMESTAMPTZ,
    CONSTRAINT chk_dlq_messages_status CHECK (status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD'))
);

-- dlq_messages_archive: アーカイブテーブル（30日経過した RESOLVED/DEAD メッセージを保管）
CREATE TABLE IF NOT EXISTS dlq.dlq_messages_archive (LIKE dlq.dlq_messages INCLUDING ALL);
```

#### featureflag-server 用スキーマ

```sql
-- infra/docker/init-db/06-featureflag-schema.sql

\c featureflag_db;

CREATE SCHEMA IF NOT EXISTS featureflag;

-- feature_flags: フィーチャーフラグ定義
CREATE TABLE IF NOT EXISTS featureflag.feature_flags (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_key    VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    enabled     BOOLEAN      NOT NULL DEFAULT false,
    variants    JSONB        NOT NULL DEFAULT '[]',
    rules       JSONB        NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- flag_evaluations: フラグ評価ログ
CREATE TABLE IF NOT EXISTS featureflag.flag_evaluations (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id     UUID         NOT NULL REFERENCES featureflag.feature_flags(id) ON DELETE CASCADE,
    user_id     VARCHAR(255),
    tenant_id   VARCHAR(255),
    result      BOOLEAN      NOT NULL,
    variant     VARCHAR(255),
    reason      VARCHAR(255),
    context     JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- flag_audit_logs: フラグ変更監査ログ
CREATE TABLE IF NOT EXISTS featureflag.flag_audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id     UUID         REFERENCES featureflag.feature_flags(id) ON DELETE SET NULL,
    flag_key    VARCHAR(255) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    changed_by  VARCHAR(255),
    old_value   JSONB,
    new_value   JSONB,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
```

#### ratelimit-server 用スキーマ

```sql
-- infra/docker/init-db/07-ratelimit-schema.sql

\c ratelimit_db;

CREATE SCHEMA IF NOT EXISTS ratelimit;

-- rate_limit_rules: レートリミットルール定義
CREATE TABLE IF NOT EXISTS ratelimit.rate_limit_rules (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) UNIQUE NOT NULL,
    key         VARCHAR(255) NOT NULL,
    limit_count BIGINT       NOT NULL,
    window_secs BIGINT       NOT NULL,
    algorithm   VARCHAR(50)  NOT NULL DEFAULT 'token_bucket',
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_rate_limit_rules_algorithm CHECK (algorithm IN ('token_bucket', 'fixed_window', 'sliding_window'))
);
```

#### tenant-server 用スキーマ

```sql
-- infra/docker/init-db/08-tenant-schema.sql

\c tenant_db;

CREATE SCHEMA IF NOT EXISTS tenant;

-- tenants: テナント管理
CREATE TABLE IF NOT EXISTS tenant.tenants (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    status       VARCHAR(50)  NOT NULL DEFAULT 'provisioning',
    plan         VARCHAR(100) NOT NULL DEFAULT 'free',
    realm_name   VARCHAR(255),
    owner_id     UUID,
    metadata     JSONB        NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_tenants_status CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted'))
);

-- tenant_members: テナントメンバー
CREATE TABLE IF NOT EXISTS tenant.tenant_members (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    user_id     UUID         NOT NULL,
    role        VARCHAR(100) NOT NULL DEFAULT 'member',
    joined_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_tenant_members_tenant_user UNIQUE (tenant_id, user_id)
);

-- tenant_provisioning_jobs: テナントプロビジョニングジョブ
CREATE TABLE IF NOT EXISTS tenant.tenant_provisioning_jobs (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    status        VARCHAR(50)  NOT NULL DEFAULT 'pending',
    current_step  VARCHAR(255),
    error_message TEXT,
    metadata      JSONB        NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_provisioning_status CHECK (status IN ('pending', 'running', 'completed', 'failed'))
);
```

#### vault-server 用スキーマ

```sql
-- infra/docker/init-db/09-vault-schema.sql

\c vault_db;

CREATE SCHEMA IF NOT EXISTS vault;

-- secret_access_logs: シークレットアクセスログ
CREATE TABLE IF NOT EXISTS vault.secret_access_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    path        VARCHAR(1024) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    subject     VARCHAR(255),
    tenant_id   VARCHAR(255),
    ip_address  INET,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    success     BOOLEAN      NOT NULL DEFAULT true,
    error_msg   TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_vault_access_action CHECK (action IN ('read', 'write', 'delete', 'list'))
);
```

### Keycloak 初期設定

Keycloak は `start-dev --import-realm` で起動し、realm 設定を自動インポートする。
Docker版では `docker-entrypoint-wrapper.sh` が envsubst で環境変数プレースホルダーを展開した後に Keycloak を起動する。

| 項目 | 設定 |
| --- | --- |
| イメージ | `quay.io/keycloak/keycloak:26.0.7` |
| Realm 名 | `k1s0` |
| Admin ユーザー | `admin` / `dev` |
| DB | PostgreSQL（`keycloak` データベース） |
| インポートパス | `./infra/docker/keycloak/` |
| ポート | `8180:8080` |
| sslRequired | `external`（Docker/K8s共通） |

#### シークレット管理

realm JSON ファイル内のシークレット（クライアントシークレット、テストユーザーパスワード）は `${VAR:-default}` 形式のプレースホルダーで定義される。

| 環境変数 | 用途 | デフォルト値 |
| --- | --- | --- |
| `KC_BFF_CLIENT_SECRET` | BFF Proxyクライアントシークレット | `dev-bff-secret` |
| `KC_SERVICE_CLIENT_SECRET` | Service-to-Serviceクライアントシークレット | `dev-service-secret` |
| `KC_TEST_ADMIN_PASSWORD` | test-adminユーザーパスワード | `admin123` |
| `KC_TEST_USER_PASSWORD` | test-userユーザーパスワード | `user123` |
| `KC_TEST_TASK_MGR_PASSWORD` | test-task-managerユーザーパスワード | `task123` |

**展開の仕組み**: `infra/docker/keycloak/docker-entrypoint-wrapper.sh` がKeycloak起動前に実行され、`envsubst`（または sed フォールバック）で realm JSON 内のプレースホルダーを環境変数値に置換する。

**本番環境**: Kubernetes では ConfigMap にシークレットを含めず、Kubernetes Secrets から環境変数として注入する。

#### Realm 設定概要

```json
// infra/docker/keycloak/k1s0-realm.json（主要部分）
{
  "realm": "k1s0",
  "enabled": true,
  "sslRequired": "external",
  "roles": {
    "realm": [
      { "name": "user", "description": "一般ユーザー" },
      { "name": "sys_admin", "description": "システム管理者" },
      { "name": "sys_operator", "description": "運用担当" },
      { "name": "sys_auditor", "description": "監査担当" },
      { "name": "biz_admin", "description": "ビジネスtier全権限" },
      { "name": "biz_operator", "description": "ビジネスtier読み書き" },
      { "name": "biz_auditor", "description": "ビジネスtier読み取り" },
      { "name": "biz_prjmst_admin", "description": "プロジェクトマスタ管理者" },
      { "name": "svc_task_admin", "description": "タスクサービス管理者" },
      { "name": "svc_internal", "description": "サービス間通信用内部ロール" },
      "..."
    ]
  },
  "clients": [
    { "clientId": "react-spa", "publicClient": true, "PKCE": "S256" },
    { "clientId": "flutter-mobile", "publicClient": true, "PKCE": "S256" },
    { "clientId": "k1s0-cli", "publicClient": true, "deviceAuth": true },
    { "clientId": "k1s0-bff", "publicClient": false, "secret": "${KC_BFF_CLIENT_SECRET:-dev-bff-secret}" },
    { "clientId": "k1s0-service", "publicClient": false, "secret": "${KC_SERVICE_CLIENT_SECRET:-dev-service-secret}", "defaultRoles": ["svc_internal"] }
  ]
}
```

#### Docker版とK8s版の同期

Docker版 (`infra/docker/keycloak/k1s0-realm.json`) を正本として管理する。K8s版は以下のファイルに同期する:

- `infra/keycloak/realm-k1s0.json` — K8s用スタンドアロンJSON
- `infra/kubernetes/verify/keycloak.yaml` — ConfigMap内の埋め込みJSON

### Kafka トピック自動作成

ローカル開発環境では、`kafka-init` コンテナでトピックを自動作成する。

```yaml
kafka-init:
  image: apache/kafka:3.8.0
  profiles: [infra]
  # セキュリティ強化: 権限昇格を禁止し、全Linuxケーパビリティを削除。読み取り専用ファイルシステムで実行
  security_opt:
    - no-new-privileges:true
  cap_drop:
    - ALL
  read_only: true
  # シェルスクリプト実行のため /tmp に exec 権限付き tmpfs をマウントする
  tmpfs:
    - /tmp:exec
  depends_on:
    kafka:
      condition: service_healthy
  environment:
    KAFKA_BOOTSTRAP_SERVER: kafka:9092
    KAFKA_REPLICATION_FACTOR: "1"
    PATH: "/opt/kafka/bin:/opt/java/openjdk/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
  volumes:
    - ./infra/messaging/kafka/create-topics.sh:/scripts/create-topics.sh:ro
  entrypoint: ["/bin/bash", "/scripts/create-topics.sh"]
  # トピック作成失敗時にリトライする（Kafka ブローカーの一時的な未準備に対応）
  restart: on-failure
```

> **注記**: トピック命名規則は `k1s0.{tier}.{domain}.{event-type}.{version}` に従う。詳細は [メッセージング設計](../../architecture/messaging/メッセージング設計.md) を参照。

### Redis

| サービス | 用途 | ポート | ボリューム |
| --- | --- | --- | --- |
| `redis` | キャッシュ / レート制限 | 6379 | `redis-data` |
| `redis-session` | BFF セッションストア | 6380 | `redis-session-data` |

## 初期化スクリプト設計

### ディレクトリ構成

```
infra/docker/
├── init-db/
│   ├── 01-create-databases.sql    # データベース作成
│   ├── 02-auth-schema.sql         # auth-server 用スキーマ
│   ├── 03-config-schema.sql       # config-server 用スキーマ
│   ├── 04-saga-schema.sql         # saga-server 用スキーマ
│   ├── 05-dlq-schema.sql          # dlq-manager 用スキーマ
│   ├── 06-featureflag-schema.sql  # featureflag-server 用スキーマ
│   ├── 07-ratelimit-schema.sql    # ratelimit-server 用スキーマ
│   ├── 08-tenant-schema.sql       # tenant-server 用スキーマ
│   ├── 09-vault-schema.sql        # vault-server 用スキーマ
│   ├── 10-project-master-schema.sql # プロジェクトマスターデータスキーマ
│   ├── 11-task-schema.sql         # タスク管理スキーマ
│   └── 12-event-store-schema.sql  # イベントストアスキーマ
├── keycloak/
│   └── k1s0-realm.json            # Keycloak realm 初期設定
├── prometheus/
│   └── prometheus.yaml            # Prometheus scrape 設定
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── datasources.yaml   # データソース自動設定
│   │   └── dashboards/
│   │       └── dashboard.yml      # ダッシュボードプロビジョニング
│   └── dashboards/
│       └── (JSON ダッシュボードファイル)
└── kong/
    └── kong.yaml                  # Kong ローカル開発用 declarative config
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
| `01-create-databases.sql` | 1 | データベース作成（keycloak, kong, k1s0_system, k1s0_business, k1s0_service） + 個別DB作成（auth_db, config_db, dlq_db, featureflag_db, ratelimit_db, tenant_db, vault_db, scheduler_db, notification_db, navigation_db, policy_db, quota_db, rule_engine_db, search_db, session_db, workflow_db, file_db）計22件 |
| `02-auth-schema.sql` | 2 | 監査ログテーブル作成（k1s0_system DB） |
| `03-config-schema.sql` | 3 | 設定値テーブル・設定変更監査ログテーブル作成（k1s0_system DB） |
| `04-saga-schema.sql` | 4 | Saga ステートマシンテーブル作成（k1s0_system DB、saga スキーマ） |
| `05-dlq-schema.sql` | 5 | DLQ メッセージ・アーカイブテーブル作成（dlq_db、dlq スキーマ） |
| `06-featureflag-schema.sql` | 6 | フィーチャーフラグ・評価ログ・監査ログテーブル作成（featureflag_db、featureflag スキーマ） |
| `07-ratelimit-schema.sql` | 7 | レートリミットルールテーブル作成（ratelimit_db、ratelimit スキーマ） |
| `08-tenant-schema.sql` | 8 | テナント・メンバー・プロビジョニングジョブテーブル作成（tenant_db、tenant スキーマ） |
| `09-vault-schema.sql` | 9 | シークレットアクセスログテーブル作成（vault_db、vault スキーマ） |
| `10-project-master-schema.sql` | 10 | プロジェクトマスターデータスキーマ |
| `11-task-schema.sql` | 11 | タスク管理スキーマ |
| `12-event-store-schema.sql` | 12 | イベントストアスキーマ |

> **注記**: 初期化スクリプトはデータボリュームが空の場合のみ実行される。既存データがある場合はスキップされるため、スキーマ変更時は `docker compose down -v` でボリュームを削除してから再起動すること。

### init-db vs マイグレーションの使い分け

<!-- M-11指摘事項: init-db と各サービスのマイグレーションの使い分け方針を明文化 -->

k1s0 では、データベーススキーマの管理を **init-db**（ローカル開発専用）と **マイグレーション**（本番・ステージング用）の2つの方式で行う。

#### init-db（`infra/docker/init-db/`）

- **用途**: ローカル開発環境の初期化専用
- **実行タイミング**: `docker compose up` 時に PostgreSQL コンテナが初回起動する際のみ
- **管理内容**: データベース作成、スキーマ作成、テーブル作成、初期データ、RLS ポリシー、権限付与
- **注意**: 本番環境には使用しない。再実行は `docker compose down -v` 後に行う

#### マイグレーション（各サービスの `database/*/migrations/`）

- **用途**: 本番環境・ステージング環境でのスキーマ変更
- **実行タイミング**: サービス起動時に `sqlx::migrate!()` で自動実行
- **管理内容**: スキーマの増分変更（カラム追加・削除、インデックス変更等）
- **注意**: init-db との二重管理は設計上の制限。将来的には init-db を廃止し migrations に統一する計画

#### 使い分けフロー

```
ローカル開発環境
  └─ docker compose up（初回）
       └─ PostgreSQL 起動 → init-db/0*.sql が自動実行 → 全スキーマ・テーブルを一括作成

本番 / ステージング環境
  └─ サービス起動
       └─ sqlx::migrate!() が自動実行 → migrations/ の未適用ファイルのみ増分適用
```

#### スキーマ変更時の手順

| 変更内容 | ローカル開発 | 本番・ステージング |
| --- | --- | --- |
| 新規テーブル追加 | init-db の該当 SQL ファイルを更新 + `docker compose down -v && up` | migrations/ にファイルを追加してデプロイ |
| カラム追加 | init-db の該当 SQL ファイルを更新 + `docker compose down -v && up` | migrations/ にファイルを追加してデプロイ |
| 初期データ変更 | init-db の該当 SQL ファイルを更新 + `docker compose down -v && up` | 別途データ移行スクリプトで対応 |

### Keycloak Realm プロビジョニング

Keycloak は `docker-entrypoint-wrapper.sh` → `start-dev --import-realm` の順で起動し、`/opt/keycloak/data/import/` にマウントされた JSON ファイルからenvsubst展開後に realm 設定を自動インポートする。

| 項目 | 設定 |
| --- | --- |
| Realm | `k1s0` |
| sslRequired | `external` |
| クライアント | `react-spa`（SPA用 PKCE）, `flutter-mobile`（Flutter用 PKCE）, `k1s0-cli`（CLI用 Device Auth）, `k1s0-bff`（BFF用）, `k1s0-service`（サービス間通信用） |
| ロール | `user`, `sys_admin`, `sys_operator`, `sys_auditor`, `biz_admin`, `biz_operator`, `biz_auditor`, `biz_prjmst_*`, `svc_task_*`, `svc_internal`, `task_manager` |
| テストユーザー | `test-admin`（sys_admin）, `test-user`（user）, `test-task-manager`（task_manager, svc_task_user） |

### Kafka トピック自動作成

`kafka-init` コンテナが Kafka ブローカーのヘルスチェック完了後に、必要なトピックを作成する。

| トピック | パーティション数 | 用途 |
| --- | --- | --- |
| `k1s0.system.auth.audit.v1` | 6 | 認証監査ログ |
| `k1s0.system.auth.permission_denied.v1` | 6 | パーミッション拒否イベント |
| `k1s0.system.config.changed.v1` | 6 | 設定変更通知 |
| `k1s0.system.apiregistry.schema_updated.v1` | 6 | API スキーマ更新通知 |
| `k1s0.system.featureflag.changed.v1` | 6 | フィーチャーフラグ変更通知 |
| `k1s0.system.file.uploaded.v1` | 6 | ファイルアップロード通知 |
| `k1s0.system.file.deleted.v1` | 6 | ファイル削除通知 |
| `k1s0.system.vault.secret_rotated.v1` | 6 | シークレットローテーション通知 |
| `k1s0.system.notification.requested.v1` | 6 | 通知リクエスト |
| `k1s0.system.quota.exceeded.v1` | 6 | クォータ超過通知 |
| `k1s0.system.saga.state_changed.v1` | 6 | Saga ステート変更イベント |
| `k1s0.service.task.created.v1` | 3 | タスク作成イベント |
| `k1s0.service.task.updated.v1` | 3 | タスク更新イベント |

---

## ポートバインディングポリシー（H-5 対応）

外部公開が不要なサービスのポートは `127.0.0.1` にバインドして、意図しない外部アクセスを防止する。

| サービス | バインディング | 理由 |
| --- | --- | --- |
| Schema Registry | `127.0.0.1:8081:8081` | 開発ツール（kafka-ui, アプリ）のみが利用。外部公開不要 |
| PostgreSQL | `127.0.0.1:5432:5432` | ローカルDBクライアントのみが利用 |
| Redis | `127.0.0.1:6379:6379` | ローカルデバッグのみが利用 |
| Kafka | `127.0.0.1:9092:9092` | ローカルプロデューサー/コンシューマーのみが利用 |

**原則**: `"PORT:PORT"` は全 NIC（`0.0.0.0`）にバインドする。共有ネットワーク（オフィス・CI）では他のホストからアクセス可能になるため、外部公開が不要なサービスは `"127.0.0.1:PORT:PORT"` を使用すること。

---

## インフラサービスのマウント・tmpfs 設計（CRIT-001/002/003 対応）

### Kafka JMX マウント

Kafka コンテナは JMX Exporter の設定ファイルを `/etc/kafka/jmx` にマウントする。
このマウントに `:ro`（読み取り専用）フラグを付けないこと（CRIT-001 対応）。

**理由**: Kafka の起動スクリプトは JMX 設定ファイルに対して `chmod 0400` を適用する。
`：ro` マウントでは chmod が失敗して Kafka ブローカーが起動できない。

```yaml
# kafka コンテナのボリューム設定
volumes:
  - ./infra/docker/kafka/jmx:/etc/kafka/jmx    # :ro は付けない（chmod 0400 が必要なため）
```

### Vault tmpfs 設計

Vault dev モードコンテナは `/tmp` と `/home/vault` の両方を tmpfs としてマウントする（CRIT-002 対応）。

**理由**: Vault dev モードは `.vault-token.tmp` ファイルを `/home/vault/` に書き込む。
`/home/vault` が tmpfs でない場合、ファイル書き込みに失敗して Vault が起動できない。

```yaml
# vault コンテナの tmpfs 設定
tmpfs:
  - /tmp
  - /home/vault    # CRIT-002: Vault dev mode token ファイル用
```

### Kong tmpfs 設計

Kong コンテナは `/tmp` と `/usr/local/kong` の両方を tmpfs としてマウントする（CRIT-003 対応）。

**理由**: Kong ワーカープロセスは Unix Domain Socket を `/usr/local/kong/` に作成する。
`/usr/local/kong` が書き込み可能でない場合、Kong ワーカーが起動できない。

```yaml
# kong コンテナの tmpfs 設定
tmpfs:
  - /tmp
  - /usr/local/kong    # CRIT-003: Kong ワーカーソケット用
```

---

## 関連ドキュメント

- [docker-compose設計.md](docker-compose設計.md) -- 基本方針・プロファイル設計
- [docker-compose-システムサービス設計.md](compose-システムサービス設計.md) -- auth-server・config-server・System プロファイル
- [docker-compose-可観測性サービス設計.md](compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Keycloak 設定
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマ
