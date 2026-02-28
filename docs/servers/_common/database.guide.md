# system-database ガイド

> **仕様**: テーブル定義・スキーマ・インデックス一覧は [database.md](./database.md) を参照。

---

## Keycloak DB との設計判断

Keycloak は自身の DB（`keycloak` データベース）でユーザー認証情報・OAuth2 設定等を管理する。auth-db はアプリケーション固有のデータを管理し、両者の責務を明確に分離する。

認証基盤（パスワード、OAuth2 クライアント、LDAP 連携）は Keycloak に委ね、アプリケーション固有のプロフィール・RBAC・監査ログ・API キーを auth-db が持つことで、認証基盤のアップグレードとアプリケーションデータの進化を独立させる。セッション管理は Redis Sentinel に配置し、BFF のステートレス性を確保する（[認証認可設計](../../architecture/auth/認証認可設計.md) 参照）。

---

## マイグレーション SQL

配置先: `regions/system/database/auth-db/migrations/`

### 001_create_schema.up.sql

```sql
-- auth-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

-- 拡張機能
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- スキーマ
CREATE SCHEMA IF NOT EXISTS auth;

-- updated_at 自動更新関数
CREATE OR REPLACE FUNCTION auth.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 001_create_schema.down.sql

```sql
DROP FUNCTION IF EXISTS auth.update_updated_at();
DROP SCHEMA IF EXISTS auth;
DROP EXTENSION IF EXISTS "pgcrypto";
```

### 002_create_users.up.sql

```sql
-- auth-db: users テーブル作成

CREATE TABLE IF NOT EXISTS auth.users (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    keycloak_sub  VARCHAR(255) UNIQUE NOT NULL,
    username      VARCHAR(255) UNIQUE NOT NULL,
    email         VARCHAR(255) UNIQUE NOT NULL,
    display_name  VARCHAR(255) NOT NULL,
    status        VARCHAR(50)  NOT NULL DEFAULT 'active',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_users_status CHECK (status IN ('active', 'inactive', 'suspended'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_users_keycloak_sub ON auth.users (keycloak_sub);
CREATE INDEX IF NOT EXISTS idx_users_status ON auth.users (status);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON auth.users (created_at);

-- updated_at トリガー
CREATE TRIGGER trigger_users_update_updated_at
    BEFORE UPDATE ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at();
```

### 002_create_users.down.sql

```sql
DROP TRIGGER IF EXISTS trigger_users_update_updated_at ON auth.users;
DROP TABLE IF EXISTS auth.users;
```

### 003_create_roles.up.sql

```sql
-- auth-db: roles テーブル作成

CREATE TABLE IF NOT EXISTS auth.roles (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    tier        VARCHAR(20)  NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_roles_tier CHECK (tier IN ('system', 'business', 'service'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_roles_tier ON auth.roles (tier);
CREATE INDEX IF NOT EXISTS idx_roles_name ON auth.roles (name);
```

### 003_create_roles.down.sql

```sql
DROP TABLE IF EXISTS auth.roles;
```

### 004_create_permissions.up.sql

```sql
-- auth-db: permissions テーブル作成

CREATE TABLE IF NOT EXISTS auth.permissions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    resource    VARCHAR(255) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    description TEXT,

    CONSTRAINT uq_permissions_resource_action UNIQUE (resource, action),
    CONSTRAINT chk_permissions_action CHECK (action IN ('read', 'write', 'delete', 'admin'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_permissions_resource ON auth.permissions (resource);
```

### 004_create_permissions.down.sql

```sql
DROP TABLE IF EXISTS auth.permissions;
```

### 005_create_user_roles_and_role_permissions.up.sql

```sql
-- auth-db: user_roles / role_permissions 中間テーブル作成

-- user_roles テーブル
CREATE TABLE IF NOT EXISTS auth.user_roles (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    role_id     UUID        NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
    assigned_by UUID        REFERENCES auth.users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_user_roles_user_role UNIQUE (user_id, role_id)
);

-- user_roles インデックス
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON auth.user_roles (user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON auth.user_roles (role_id);

-- role_permissions テーブル
CREATE TABLE IF NOT EXISTS auth.role_permissions (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id       UUID        NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
    permission_id UUID        NOT NULL REFERENCES auth.permissions(id) ON DELETE CASCADE,
    granted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_role_permissions_role_permission UNIQUE (role_id, permission_id)
);

-- role_permissions インデックス
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON auth.role_permissions (role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON auth.role_permissions (permission_id);
```

### 005_create_user_roles_and_role_permissions.down.sql

```sql
DROP TABLE IF EXISTS auth.role_permissions;
DROP TABLE IF EXISTS auth.user_roles;
```

### 006_create_audit_logs.up.sql

```sql
-- auth-db: audit_logs テーブル作成（月次パーティショニング）

CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id          UUID         NOT NULL DEFAULT gen_random_uuid(),
    user_id     TEXT,
    event_type  VARCHAR(100) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    resource_id VARCHAR(255),
    result      VARCHAR(50)  NOT NULL DEFAULT 'SUCCESS',
    detail      JSONB,
    ip_address  TEXT,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id_created_at
    ON auth.audit_logs (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type_created_at
    ON auth.audit_logs (event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at
    ON auth.audit_logs (action, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_trace_id
    ON auth.audit_logs (trace_id)
    WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON auth.audit_logs (resource, resource_id)
    WHERE resource IS NOT NULL;

-- 初期パーティション（直近3ヶ月 + 将来3ヶ月）
-- 本番運用では cron ジョブまたは pg_partman で自動作成する
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_01 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_02 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_03 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_04 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_05 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_06 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

-- デフォルトパーティション（パーティションが存在しない範囲のデータを受け付ける）
CREATE TABLE IF NOT EXISTS auth.audit_logs_default PARTITION OF auth.audit_logs DEFAULT;
```

### 006_create_audit_logs.down.sql

```sql
DROP TABLE IF EXISTS auth.audit_logs;
```

### 007_create_api_keys.up.sql

```sql
-- auth-db: api_keys テーブル作成

CREATE TABLE IF NOT EXISTS auth.api_keys (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL,
    key_hash     VARCHAR(255) UNIQUE NOT NULL,
    key_prefix   VARCHAR(10)  NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    tier         VARCHAR(20)  NOT NULL,
    permissions  JSONB        NOT NULL DEFAULT '[]',
    expires_at   TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    is_active    BOOLEAN      NOT NULL DEFAULT true,
    created_by   UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_api_keys_tier CHECK (tier IN ('system', 'business', 'service'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_service_name ON auth.api_keys (service_name);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON auth.api_keys (key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON auth.api_keys (expires_at) WHERE expires_at IS NOT NULL;

-- updated_at トリガー
CREATE TRIGGER trigger_api_keys_update_updated_at
    BEFORE UPDATE ON auth.api_keys
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at();
```

### 007_create_api_keys.down.sql

```sql
DROP TRIGGER IF EXISTS trigger_api_keys_update_updated_at ON auth.api_keys;
DROP TABLE IF EXISTS auth.api_keys;
```

### 008_seed_initial_data.up.sql

初期データ投入（デフォルトロール・権限・ロール権限マッピング）。以前は seeds/ ディレクトリに配置していたが、マイグレーションに統合した。

```sql
-- auth-db: 初期データ投入（デフォルトロール・権限・ロール権限マッピング）

-- デフォルトロール
INSERT INTO auth.roles (name, description, tier) VALUES
    ('sys_admin',    'システム全体の管理者。すべてのリソースに対する全権限',         'system'),
    ('sys_operator', 'システム運用担当。監視・ログ閲覧・設定変更',                   'system'),
    ('sys_auditor',  '監査担当。全リソースの読み取り専用',                            'system')
ON CONFLICT (name) DO NOTHING;

-- デフォルト権限
INSERT INTO auth.permissions (resource, action, description) VALUES
    ('users',        'read',   'ユーザー情報の閲覧'),
    ('users',        'write',  'ユーザー情報の作成・更新'),
    ('users',        'delete', 'ユーザーの削除'),
    ('users',        'admin',  'ユーザー管理の全権限'),
    ('auth_config',  'read',   '認証設定の閲覧'),
    ('auth_config',  'write',  '認証設定の作成・更新'),
    ('auth_config',  'delete', '認証設定の削除'),
    ('auth_config',  'admin',  '認証設定管理の全権限'),
    ('audit_logs',   'read',   '監査ログの閲覧'),
    ('api_gateway',  'read',   'API Gateway 設定の閲覧'),
    ('api_gateway',  'write',  'API Gateway 設定の作成・更新'),
    ('api_gateway',  'delete', 'API Gateway 設定の削除'),
    ('api_gateway',  'admin',  'API Gateway 管理の全権限'),
    ('vault_secrets','read',   'Vault シークレットの閲覧'),
    ('vault_secrets','write',  'Vault シークレットの作成・更新'),
    ('vault_secrets','delete', 'Vault シークレットの削除'),
    ('vault_secrets','admin',  'Vault シークレット管理の全権限'),
    ('monitoring',   'read',   '監視データの閲覧'),
    ('monitoring',   'write',  '監視設定の作成・更新'),
    ('monitoring',   'delete', '監視設定の削除'),
    ('monitoring',   'admin',  '監視管理の全権限')
ON CONFLICT (resource, action) DO NOTHING;

-- デフォルトロール・権限マッピング（sys_admin / sys_operator / sys_auditor）
-- 省略（詳細は Seeds セクション参照）
```

### 008_seed_initial_data.down.sql

```sql
DELETE FROM auth.role_permissions;
DELETE FROM auth.permissions;
DELETE FROM auth.roles WHERE name IN ('sys_admin', 'sys_operator', 'sys_auditor');
```

### 009_align_audit_log_columns.up.sql

audit_logs テーブルのカラム名を変更（`detail` → `metadata`、`created_at` → `recorded_at`）。

```sql
ALTER TABLE auth.audit_logs RENAME COLUMN detail TO metadata;
ALTER TABLE auth.audit_logs RENAME COLUMN created_at TO recorded_at;
```

### 009_align_audit_log_columns.down.sql

```sql
ALTER TABLE auth.audit_logs RENAME COLUMN metadata TO detail;
ALTER TABLE auth.audit_logs RENAME COLUMN recorded_at TO created_at;
```

### 010_fix_audit_log_columns.up.sql

009 で変更されたカラム名を正規設計（system-database.md）に戻す。

```sql
ALTER TABLE auth.audit_logs RENAME COLUMN metadata TO detail;
ALTER TABLE auth.audit_logs RENAME COLUMN recorded_at TO created_at;
```

### 010_fix_audit_log_columns.down.sql

```sql
ALTER TABLE auth.audit_logs RENAME COLUMN detail TO metadata;
ALTER TABLE auth.audit_logs RENAME COLUMN created_at TO recorded_at;
```

### 011_create_partition_management.up.sql

pg_partman 拡張を使用した audit_logs テーブルの月次パーティション自動管理。pg_partman が利用できない環境（テストコンテナ等）では自動的にスキップする。

```sql
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_available_extensions WHERE name = 'pg_partman'
    ) THEN
        CREATE EXTENSION IF NOT EXISTS pg_partman SCHEMA partman;

        PERFORM partman.create_parent(
            p_parent_table   := 'auth.audit_logs',
            p_control        := 'created_at',
            p_type           := 'native',
            p_interval       := '1 month',
            p_premake        := 3
        );

        UPDATE partman.part_config
        SET
            retention                = '24 months',
            retention_keep_table     = false,
            retention_keep_index     = false,
            automatic_maintenance    = 'on',
            infinite_time_partitions = true
        WHERE parent_table = 'auth.audit_logs';

        PERFORM partman.run_maintenance(p_parent_table := 'auth.audit_logs');
    ELSE
        RAISE NOTICE 'pg_partman is not available; skipping partition management setup.';
    END IF;
END $$;
```

### 011_create_partition_management.down.sql

```sql
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_extension WHERE extname = 'pg_partman'
    ) THEN
        DELETE FROM partman.part_config WHERE parent_table = 'auth.audit_logs';
        DROP EXTENSION IF EXISTS pg_partman;
    END IF;
END $$;
```

---

## Seeds（初期データ）

> **注**: 初期データ投入は `008_seed_initial_data` マイグレーションに統合済み。以下は参照用。

配置先: `regions/system/database/auth-db/seeds/`

### デフォルトロール

[認証認可設計](../../architecture/auth/認証認可設計.md) D-005 の Tier 別ロール定義に対応する初期ロールを投入する。

```sql
-- seeds/001_default_roles.sql

-- system Tier ロール
INSERT INTO auth.roles (name, description, tier) VALUES
    ('sys_admin',    'システム全体の管理者。すべてのリソースに対する全権限',         'system'),
    ('sys_operator', 'システム運用担当。監視・ログ閲覧・設定変更',                   'system'),
    ('sys_auditor',  '監査担当。全リソースの読み取り専用',                            'system')
ON CONFLICT (name) DO NOTHING;
```

### デフォルト権限

[認証認可設計](../../architecture/auth/認証認可設計.md) D-005 のパーミッションマトリクスに対応する初期権限を投入する。

```sql
-- seeds/002_default_permissions.sql

-- system Tier リソースの権限
INSERT INTO auth.permissions (resource, action, description) VALUES
    -- users リソース
    ('users',        'read',   'ユーザー情報の閲覧'),
    ('users',        'write',  'ユーザー情報の作成・更新'),
    ('users',        'delete', 'ユーザーの削除'),
    ('users',        'admin',  'ユーザー管理の全権限'),
    -- auth_config リソース
    ('auth_config',  'read',   '認証設定の閲覧'),
    ('auth_config',  'write',  '認証設定の作成・更新'),
    ('auth_config',  'delete', '認証設定の削除'),
    ('auth_config',  'admin',  '認証設定管理の全権限'),
    -- audit_logs リソース
    ('audit_logs',   'read',   '監査ログの閲覧'),
    -- api_gateway リソース
    ('api_gateway',  'read',   'API Gateway 設定の閲覧'),
    ('api_gateway',  'write',  'API Gateway 設定の作成・更新'),
    ('api_gateway',  'delete', 'API Gateway 設定の削除'),
    ('api_gateway',  'admin',  'API Gateway 管理の全権限'),
    -- vault_secrets リソース
    ('vault_secrets','read',   'Vault シークレットの閲覧'),
    ('vault_secrets','write',  'Vault シークレットの作成・更新'),
    ('vault_secrets','delete', 'Vault シークレットの削除'),
    ('vault_secrets','admin',  'Vault シークレット管理の全権限'),
    -- monitoring リソース
    ('monitoring',   'read',   '監視データの閲覧'),
    ('monitoring',   'write',  '監視設定の作成・更新'),
    ('monitoring',   'delete', '監視設定の削除'),
    ('monitoring',   'admin',  '監視管理の全権限')
ON CONFLICT (resource, action) DO NOTHING;
```

### デフォルトロール・権限マッピング

[認証認可設計](../../architecture/auth/認証認可設計.md) D-005 の system Tier パーミッションマトリクスに対応する。

```sql
-- seeds/003_default_role_permissions.sql

-- sys_admin: すべてのリソースに対する全権限
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- sys_operator: 監視・ログ閲覧・設定変更
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_operator'
  AND (
    (p.resource = 'users'        AND p.action = 'read')
    OR (p.resource = 'auth_config'  AND p.action IN ('read', 'write'))
    OR (p.resource = 'audit_logs'   AND p.action = 'read')
    OR (p.resource = 'api_gateway'  AND p.action = 'read')
    OR (p.resource = 'vault_secrets' AND p.action = 'read')
    OR (p.resource = 'monitoring'   AND p.action IN ('read', 'write'))
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- sys_auditor: 全リソースの読み取り専用（vault_secrets を除く）
INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_auditor'
  AND p.action = 'read'
  AND p.resource != 'vault_secrets'
ON CONFLICT (role_id, permission_id) DO NOTHING;
```

---

## インデックス設計方針

- **部分インデックス**: NULL 値が多いカラム（trace_id, resource, expires_at）やフラグ（is_active）には部分インデックスを使用し、インデックスサイズを削減する
- **複合インデックス**: 時系列検索が主要ユースケースとなる audit_logs では、フィルタ条件 + created_at の複合インデックスを定義する
- **UNIQUE 制約**: keycloak_sub, username, email, key_hash は UNIQUE 制約により暗黙的にインデックスが作成されるが、部分インデックス等の追加最適化が必要な場合は別途定義する

---

## パーティショニング設計背景

audit_logs テーブルは月次レンジパーティショニングを適用する。監査ログは書き込みが多く長期保存が必要なため、パーティショニングにより以下の効果を得る。

| 効果 | 説明 |
|------|------|
| クエリパフォーマンス | created_at による範囲検索でパーティションプルーニングが効く |
| メンテナンス | 古いパーティションの DROP による高速な大量データ削除 |
| VACUUM 負荷の分散 | パーティション単位で VACUUM が実行されるため、テーブル全体のロックを回避 |
| アーカイブ | 古いパーティションを別テーブルスペースに移動し、コスト最適化 |

### パーティション作成例

```sql
-- 月次パーティション作成（cron ジョブまたは pg_partman で自動化）
-- 3ヶ月先のパーティションを事前に作成しておく

-- 新規パーティション作成の例
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_07 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');
```

### アーカイブ手順

```sql
-- 1. パーティションを切り離す
ALTER TABLE auth.audit_logs DETACH PARTITION auth.audit_logs_2024_01;

-- 2. バックアップ（pg_dump）
-- pg_dump -t auth.audit_logs_2024_01 k1s0_system > audit_logs_2024_01.sql

-- 3. テーブルを削除（バックアップ確認後）
DROP TABLE auth.audit_logs_2024_01;
```

---

## 接続設定の実装例

[config設計](../../cli/config/config設計.md) の database セクションに従い、auth-db への接続を以下のように設定する。

### config.yaml（auth サーバー用）

```yaml
# config/config.yaml — auth サーバー
app:
  name: "auth-server"
  version: "1.0.0"
  tier: "system"
  environment: "dev"

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""                   # Vault パス: secret/data/k1s0/system/auth/database キー: password
  ssl_mode: "disable"            # dev 環境。staging: require、prod: verify-full
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
```

### docker-compose（ローカル開発）

[docker-compose設計](../../infrastructure/docker/docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを作成する。初期化スクリプトは `infra/docker/init-db/` に配置する。

```sql
-- infra/docker/init-db/01-create-databases.sql（抜粋）
CREATE DATABASE k1s0_system;
```

---

## 主要クエリパターン

auth-db で頻繁に実行されるクエリパターンとそのインデックス活用を以下に示す。

### ユーザー認証・認可

```sql
-- JWT の sub からユーザー情報を取得
SELECT id, username, email, display_name, status
FROM auth.users
WHERE keycloak_sub = $1 AND status = 'active';

-- ユーザーのロール一覧を取得
SELECT r.name, r.tier
FROM auth.roles r
INNER JOIN auth.user_roles ur ON ur.role_id = r.id
WHERE ur.user_id = $1;

-- ユーザーの権限一覧を取得（ロール経由）
SELECT DISTINCT p.resource, p.action
FROM auth.permissions p
INNER JOIN auth.role_permissions rp ON rp.permission_id = p.id
INNER JOIN auth.user_roles ur ON ur.role_id = rp.role_id
WHERE ur.user_id = $1;
```

### 監査ログ

```sql
-- ユーザーの操作履歴を取得（時系列）
SELECT id, action, resource, resource_id, detail, ip_address, created_at
FROM auth.audit_logs
WHERE user_id = $1
  AND created_at BETWEEN $2 AND $3
ORDER BY created_at DESC
LIMIT $4 OFFSET $5;

-- トレース ID による検索（OpenTelemetry 連携）
SELECT id, user_id, action, resource, detail, created_at
FROM auth.audit_logs
WHERE trace_id = $1;
```

### API キー認証

```sql
-- API キーのハッシュで認証
SELECT id, name, service_name, tier, permissions, expires_at
FROM auth.api_keys
WHERE key_hash = $1
  AND is_active = true
  AND (expires_at IS NULL OR expires_at > NOW());

-- API キーの最終使用日時を更新
UPDATE auth.api_keys
SET last_used_at = NOW()
WHERE id = $1;
```

---

## バックアップ・リストア

### バックアップ実行例

```bash
# フルバックアップ（pg_basebackup）
pg_basebackup -h postgres.k1s0-system.svc.cluster.local -U replication -D /backup/base -Ft -z -P

# 論理バックアップ（pg_dump）
pg_dump -h postgres.k1s0-system.svc.cluster.local -U app -d k1s0_system -Fc -f /backup/k1s0_system.dump
```
