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
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.auth.permission_denied.v1 --partitions 6 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.config.changed.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.apiregistry.schema_updated.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.featureflag.changed.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.file.uploaded.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.file.deleted.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.vault.secret_rotated.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.notification.requested.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.system.quota.exceeded.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.service.order.created.v1 --partitions 3 --replication-factor 1
      kafka-topics.sh --bootstrap-server kafka:9092 --create --if-not-exists --topic k1s0.service.order.updated.v1 --partitions 3 --replication-factor 1
      echo "Kafka topics created."
  restart: "no"
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
| `01-create-databases.sql` | 1 | データベース作成（keycloak, kong, k1s0_system, k1s0_business, k1s0_service） + 個別DB作成（auth_db, config_db, dlq_db） |
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
| `k1s0.system.auth.permission_denied.v1` | 6 | パーミッション拒否イベント |
| `k1s0.system.config.changed.v1` | 3 | 設定変更通知 |
| `k1s0.system.apiregistry.schema_updated.v1` | 3 | API スキーマ更新通知 |
| `k1s0.system.featureflag.changed.v1` | 3 | フィーチャーフラグ変更通知 |
| `k1s0.system.file.uploaded.v1` | 3 | ファイルアップロード通知 |
| `k1s0.system.file.deleted.v1` | 3 | ファイル削除通知 |
| `k1s0.system.vault.secret_rotated.v1` | 3 | シークレットローテーション通知 |
| `k1s0.system.notification.requested.v1` | 3 | 通知リクエスト |
| `k1s0.system.quota.exceeded.v1` | 3 | クォータ超過通知 |
| `k1s0.service.order.created.v1` | 3 | 注文作成イベント |
| `k1s0.service.order.updated.v1` | 3 | 注文更新イベント |

---

## 関連ドキュメント

- [docker-compose設計.md](docker-compose設計.md) -- 基本方針・プロファイル設計
- [docker-compose-システムサービス設計.md](compose-システムサービス設計.md) -- auth-server・config-server・System プロファイル
- [docker-compose-可観測性サービス設計.md](compose-可観測性サービス設計.md) -- Prometheus・Grafana・Loki・Jaeger の詳細設定
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Keycloak 設定
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマ
