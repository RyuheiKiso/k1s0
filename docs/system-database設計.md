# system-database設計

system Tier の認証・認可データベース（auth-db）の設計を定義する。
配置先: `regions/system/database/auth-db/`

## 概要

auth-db は system Tier に属する PostgreSQL 17 データベースであり、アプリケーションレベルの認証・認可データを管理する。Keycloak が管理する認証情報とは独立し、アプリケーション固有のユーザープロフィール、RBAC マッピング、監査ログ、API キーを保持する。

[tier-architecture.md](tier-architecture.md) の設計原則に従い、auth-db へのアクセスは **system Tier のサーバーからのみ** 許可する。下位 Tier（business / service）がユーザー情報や権限情報を必要とする場合は、system Tier のサーバーが提供する API（gRPC 等）を経由する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション（Go） | golang-migrate | - |
| マイグレーション（Rust） | sqlx-cli | - |
| ORM / クエリビルダー | sqlx（Go / Rust 共通） | - |
| シークレット管理 | HashiCorp Vault | 1.17 |

### Keycloak DB との役割分担

Keycloak は自身の DB（`keycloak` データベース）でユーザー認証情報・OAuth2 設定等を管理する。auth-db はアプリケーション固有のデータを管理し、両者の責務を明確に分離する。

| データ | 管理先 | 理由 |
|--------|--------|------|
| ユーザー認証情報（パスワード等） | Keycloak DB | Keycloak が管理する認証基盤のデータ |
| OAuth2 クライアント設定 | Keycloak DB | Keycloak が管理するプロトコル設定 |
| LDAP / AD 連携設定 | Keycloak DB | Keycloak の User Federation 機能 |
| セッション管理 | Redis Sentinel | BFF セッションストア（[認証認可設計](認証認可設計.md) 参照） |
| ユーザープロフィール（アプリ固有） | auth-db | Keycloak の sub と紐づくアプリケーション固有データ |
| ロール・権限マッピング（アプリ固有） | auth-db | [認証認可設計](認証認可設計.md) D-005 の細粒度 RBAC |
| 監査ログ | auth-db | 長期保存・全文検索・コンプライアンス対応 |
| API キー | auth-db | サービス間認証のフォールバック |

---

## ER図

```
┌─────────────┐       ┌──────────────────┐       ┌─────────────────┐
│   users     │       │   user_roles     │       │     roles       │
├─────────────┤       ├──────────────────┤       ├─────────────────┤
│ id (PK)     │──┐    │ id (PK)          │    ┌──│ id (PK)         │
│ keycloak_sub│  └───>│ user_id (FK)     │    │  │ name            │
│ username    │       │ role_id (FK)     │<───┘  │ description     │
│ email       │       │ assigned_by      │       │ tier            │
│ display_name│       │ assigned_at      │       │ created_at      │
│ status      │       └──────────────────┘       └─────────────────┘
│ created_at  │                                          │
│ updated_at  │                                          │
└─────────────┘       ┌──────────────────┐               │
       │              │ role_permissions  │               │
       │              ├──────────────────┤       ┌───────┘
       │              │ id (PK)          │       │
       │              │ role_id (FK)     │<──────┘
       │              │ permission_id(FK)│───┐
       │              │ granted_at       │   │
       │              └──────────────────┘   │
       │                                     │
       │              ┌──────────────────┐   │
       │              │  permissions     │   │
       │              ├──────────────────┤   │
       │              │ id (PK)          │<──┘
       │              │ resource         │
       │              │ action           │
       │              │ description      │
       │              └──────────────────┘
       │
       │              ┌──────────────────────┐
       │              │  audit_logs          │
       │              │  (user_id は TEXT、   │
       │              │   FK なし)            │
       │              ├──────────────────────┤
       │              │ id (PK composite)    │
       │              │ user_id (TEXT)       │
       │              │ event_type           │
       │              │ action               │
       │              │ resource             │
       │              │ resource_id          │
       │              │ result               │
       │              │ detail (JSONB)       │
       │              │ ip_address (TEXT)    │
       │              │ user_agent           │
       │              │ trace_id             │
       │              │ created_at (PK comp) │
       │              └──────────────────────┘

┌─────────────────┐
│   api_keys      │
├─────────────────┤
│ id (PK)         │
│ name            │
│ key_hash        │
│ key_prefix      │
│ service_name    │
│ tier            │
│ permissions     │
│ expires_at      │
│ last_used_at    │
│ is_active       │
│ created_by (FK) │──> users(id)
│ created_at      │
│ updated_at      │
└─────────────────┘
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| users - user_roles | 1:N | ユーザーは複数のロールを持てる |
| roles - user_roles | 1:N | ロールは複数のユーザーに割り当てられる |
| roles - role_permissions | 1:N | ロールは複数の権限を持てる |
| permissions - role_permissions | 1:N | 権限は複数のロールに付与される |
| users - audit_logs | 1:N（論理的。FK なし） | ユーザーは複数の監査ログを生成する（user_id は TEXT 型で FK 制約なし） |
| users - api_keys | 1:N | ユーザーは複数の API キーを作成できる |

---

## テーブル定義

### users テーブル

Keycloak の `sub` claim（UUID）と紐づくアプリケーション固有のユーザーデータを管理する。[認証認可設計](認証認可設計.md) の JWT Claims 構造で定義された `sub` を `keycloak_sub` カラムで参照する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | アプリケーション内部の識別子 |
| keycloak_sub | VARCHAR(255) | UNIQUE NOT NULL | Keycloak の sub claim（ユーザーの一意識別子） |
| username | VARCHAR(255) | UNIQUE NOT NULL | ログイン ID（Keycloak の preferred_username と同期） |
| email | VARCHAR(255) | UNIQUE NOT NULL | メールアドレス（Keycloak の email と同期） |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| status | VARCHAR(50) | NOT NULL DEFAULT 'active' | active / inactive / suspended |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時（トリガーで自動更新） |

### roles テーブル

[認証認可設計](認証認可設計.md) D-005 で定義された Tier 別ロール（`sys_admin`, `biz_{domain}_admin`, `svc_{service}_admin` 等）を管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | ロール識別子 |
| name | VARCHAR(100) | UNIQUE NOT NULL | ロール名（例: sys_admin, svc_order_user） |
| description | TEXT | | ロールの説明 |
| tier | VARCHAR(20) | NOT NULL | system / business / service |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |

### permissions テーブル

リソースに対する操作権限を管理する。[認証認可設計](認証認可設計.md) D-005 のパーミッションマトリクスに対応する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | 権限識別子 |
| resource | VARCHAR(255) | NOT NULL | 操作対象リソース（例: orders, users, config） |
| action | VARCHAR(50) | NOT NULL | 操作種別（read, write, delete, admin） |
| description | TEXT | | 権限の説明 |
| | | UNIQUE(resource, action) | リソースと操作の組み合わせで一意 |

### user_roles テーブル（多対多）

ユーザーとロールの割り当てを管理する中間テーブル。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | レコード識別子 |
| user_id | UUID | FK users(id) ON DELETE CASCADE, NOT NULL | ユーザー ID |
| role_id | UUID | FK roles(id) ON DELETE CASCADE, NOT NULL | ロール ID |
| assigned_by | UUID | FK users(id) ON DELETE SET NULL | 割り当てた管理者の ID |
| assigned_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 割り当て日時 |
| | | UNIQUE(user_id, role_id) | 同一ユーザーに同一ロールは1回のみ |

### role_permissions テーブル（多対多）

ロールと権限の関連付けを管理する中間テーブル。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | レコード識別子 |
| role_id | UUID | FK roles(id) ON DELETE CASCADE, NOT NULL | ロール ID |
| permission_id | UUID | FK permissions(id) ON DELETE CASCADE, NOT NULL | 権限 ID |
| granted_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 付与日時 |
| | | UNIQUE(role_id, permission_id) | 同一ロールに同一権限は1回のみ |

### audit_logs テーブル

ユーザーの操作履歴を記録する監査ログテーブル。OpenTelemetry の trace_id を記録し、[可観測性設計](可観測性設計.md) の分散トレーシングと連携する。月次パーティショニングにより大量データを効率的に管理する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | NOT NULL, DEFAULT gen_random_uuid() | ログ識別子 |
| user_id | TEXT | | 操作ユーザーの ID（文字列型。FK なし） |
| event_type | VARCHAR(100) | NOT NULL | イベント種別（LOGIN_SUCCESS, LOGIN_FAILURE, LOGOUT, PERMISSION_CHANGE 等） |
| action | VARCHAR(100) | NOT NULL | 操作種別（login, logout, permission_change, user_create, user_update, role_assign, api_key_create 等） |
| resource | VARCHAR(255) | | 操作対象リソース種別 |
| resource_id | VARCHAR(255) | | 操作対象リソースの ID |
| result | VARCHAR(50) | NOT NULL DEFAULT 'SUCCESS' | 操作結果（SUCCESS, FAILURE, DENIED） |
| detail | JSONB | | 操作の詳細情報（変更前後の値等） |
| ip_address | TEXT | | クライアント IP アドレス（TEXT 型で IPv4/IPv6 文字列を柔軟に格納） |
| user_agent | TEXT | | クライアント User-Agent |
| trace_id | VARCHAR(64) | | OpenTelemetry トレース ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 記録日時 |
| | | PRIMARY KEY (id, created_at) | パーティションキーを含む複合主キー |

> **注意**: PostgreSQL のパーティショニングでは、パーティションキー（`created_at`）を PRIMARY KEY に含める必要があるため、`PRIMARY KEY (id, created_at)` の複合主キーを使用する。`user_id` は `TEXT` 型（users テーブルへの FK なし）であり、Keycloak sub やシステムアカウント名など柔軟な識別子を格納できる。`ip_address` は `TEXT` 型であり、INET 型ではない。

**インデックス**: `(user_id, created_at)`, `(event_type, created_at)`, `(action, created_at)`, `(trace_id WHERE NOT NULL)`, `(resource, resource_id WHERE NOT NULL)`

### api_keys テーブル

サービス間認証のフォールバックとして使用する API キーを管理する。mTLS + Client Credentials が利用できない場合の代替手段。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK, DEFAULT gen_random_uuid() | API キー識別子 |
| name | VARCHAR(255) | NOT NULL | API キーの名前（識別用） |
| key_hash | VARCHAR(255) | UNIQUE NOT NULL | API キーの SHA-256 ハッシュ |
| key_prefix | VARCHAR(10) | NOT NULL | キーの先頭8文字（識別・ログ表示用） |
| service_name | VARCHAR(255) | NOT NULL | 使用するサービス名 |
| tier | VARCHAR(20) | NOT NULL | system / business / service |
| permissions | JSONB | NOT NULL DEFAULT '[]' | 許可する操作の一覧（JSON 配列） |
| expires_at | TIMESTAMPTZ | | 有効期限（NULL の場合は無期限） |
| last_used_at | TIMESTAMPTZ | | 最終使用日時 |
| is_active | BOOLEAN | NOT NULL DEFAULT true | 有効フラグ |
| created_by | UUID | FK users(id) ON DELETE SET NULL | 作成者の ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

---

## マイグレーションファイル

配置先: `regions/system/database/auth-db/migrations/`

命名規則は [テンプレート仕様-データベース](テンプレート仕様-データベース.md) に準拠する。

```
migrations/
├── 001_create_schema.up.sql                            # スキーマ・拡張機能・共通関数
├── 001_create_schema.down.sql
├── 002_create_users.up.sql                             # users テーブル
├── 002_create_users.down.sql
├── 003_create_roles.up.sql                             # roles テーブル
├── 003_create_roles.down.sql
├── 004_create_permissions.up.sql                       # permissions テーブル
├── 004_create_permissions.down.sql
├── 005_create_user_roles_and_role_permissions.up.sql   # user_roles + role_permissions 中間テーブル
├── 005_create_user_roles_and_role_permissions.down.sql
├── 006_create_audit_logs.up.sql                        # audit_logs テーブル（月次パーティション）
├── 006_create_audit_logs.down.sql
├── 007_create_api_keys.up.sql                          # api_keys テーブル
├── 007_create_api_keys.down.sql
├── 008_seed_initial_data.up.sql                        # 初期データ投入（ロール・権限・マッピング）
├── 008_seed_initial_data.down.sql
├── 009_align_audit_log_columns.up.sql                  # audit_logs カラム名変更（detail→metadata, created_at→recorded_at）
├── 009_align_audit_log_columns.down.sql
├── 010_fix_audit_log_columns.up.sql                    # audit_logs カラム名を正規設計に戻す（metadata→detail, recorded_at→created_at）
├── 010_fix_audit_log_columns.down.sql
├── 011_create_partition_management.up.sql              # pg_partman による自動パーティション管理
└── 011_create_partition_management.down.sql
```

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

009 で変更されたカラム名を正規設計（system-database設計.md）に戻す。

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

[認証認可設計](認証認可設計.md) D-005 の Tier 別ロール定義に対応する初期ロールを投入する。

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

[認証認可設計](認証認可設計.md) D-005 のパーミッションマトリクスに対応する初期権限を投入する。

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

[認証認可設計](認証認可設計.md) D-005 の system Tier パーミッションマトリクスに対応する。

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

## インデックス設計

パフォーマンスを確保するためのインデックス設計方針を以下に整理する。

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| users | idx_users_keycloak_sub | keycloak_sub | B-tree | JWT の sub からのユーザー検索（UNIQUE 制約とは別にパフォーマンス用） |
| users | idx_users_status | status | B-tree | ステータスによるフィルタリング |
| users | idx_users_created_at | created_at | B-tree | 作成日時による範囲検索・ソート |
| roles | idx_roles_tier | tier | B-tree | Tier によるロール検索 |
| roles | idx_roles_name | name | B-tree | ロール名による検索 |
| permissions | idx_permissions_resource | resource | B-tree | リソースによる権限検索 |
| user_roles | idx_user_roles_user_id | user_id | B-tree | ユーザーに紐づくロール取得 |
| user_roles | idx_user_roles_role_id | role_id | B-tree | ロールに紐づくユーザー取得 |
| role_permissions | idx_role_permissions_role_id | role_id | B-tree | ロールに紐づく権限取得 |
| role_permissions | idx_role_permissions_permission_id | permission_id | B-tree | 権限に紐づくロール取得 |
| audit_logs | idx_audit_logs_user_id_created_at | (user_id, created_at) | B-tree | ユーザー別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_event_type_created_at | (event_type, created_at) | B-tree | イベント種別別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_action_created_at | (action, created_at) | B-tree | 操作種別別の監査ログ時系列検索 |
| audit_logs | idx_audit_logs_trace_id | trace_id (WHERE NOT NULL) | B-tree (部分) | OpenTelemetry トレース ID による検索 |
| audit_logs | idx_audit_logs_resource | (resource, resource_id) (WHERE NOT NULL) | B-tree (部分) | リソース種別・ID による検索 |
| api_keys | idx_api_keys_key_hash | key_hash (WHERE is_active) | B-tree (部分) | API キー認証時のハッシュ検索（アクティブのみ） |
| api_keys | idx_api_keys_service_name | service_name | B-tree | サービス名による API キー検索 |
| api_keys | idx_api_keys_key_prefix | key_prefix | B-tree | プレフィックスによる API キー識別 |
| api_keys | idx_api_keys_expires_at | expires_at (WHERE NOT NULL) | B-tree (部分) | 期限切れ API キーの検出 |

### 設計方針

- **部分インデックス**: NULL 値が多いカラム（trace_id, resource, expires_at）やフラグ（is_active）には部分インデックスを使用し、インデックスサイズを削減する
- **複合インデックス**: 時系列検索が主要ユースケースとなる audit_logs では、フィルタ条件 + created_at の複合インデックスを定義する
- **UNIQUE 制約**: keycloak_sub, username, email, key_hash は UNIQUE 制約により暗黙的にインデックスが作成されるが、部分インデックス等の追加最適化が必要な場合は別途定義する

---

## パーティショニング

### audit_logs テーブルのパーティショニング

audit_logs テーブルは月次レンジパーティショニングを適用する。監査ログは書き込みが多く長期保存が必要なため、パーティショニングにより以下の効果を得る。

| 効果 | 説明 |
|------|------|
| クエリパフォーマンス | created_at による範囲検索でパーティションプルーニングが効く |
| メンテナンス | 古いパーティションの DROP による高速な大量データ削除 |
| VACUUM 負荷の分散 | パーティション単位で VACUUM が実行されるため、テーブル全体のロックを回避 |
| アーカイブ | 古いパーティションを別テーブルスペースに移動し、コスト最適化 |

### パーティション管理

```sql
-- 月次パーティション作成（cron ジョブまたは pg_partman で自動化）
-- 3ヶ月先のパーティションを事前に作成しておく

-- 新規パーティション作成の例
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_07 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');
```

### パーティション運用ルール

| 項目 | ルール |
|------|--------|
| パーティション単位 | 月次（1ヶ月 = 1パーティション） |
| 事前作成 | 3ヶ月先のパーティションを常に準備 |
| 保持期間 | 24ヶ月（コンプライアンス要件に応じて調整） |
| アーカイブ | 保持期間超過のパーティションは DETACH 後にアーカイブストレージへ移動 |
| デフォルトパーティション | 範囲外データの受け皿として常に存在させる |
| 自動化 | pg_partman または cron ジョブでパーティション作成・削除を自動化 |

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

## 接続設定

[config設計](config設計.md) の database セクションに従い、auth-db への接続を以下のように設定する。

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

### 環境別設定

| 環境 | host | ssl_mode | max_open_conns | max_idle_conns |
|------|------|----------|----------------|----------------|
| dev | localhost (docker-compose) | disable | 10 | 3 |
| staging | postgres.k1s0-system.svc.cluster.local | require | 25 | 5 |
| prod | postgres.k1s0-system.svc.cluster.local | verify-full | 50 | 10 |

### Vault によるクレデンシャル管理

[認証認可設計](認証認可設計.md) D-006 のシークレットパス体系に従い、以下の Vault パスから DB クレデンシャルを取得する。

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/system/auth/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/system-auth-rw` | Vault Database エンジンで自動生成（TTL: 24時間） |
| 動的クレデンシャル（読み取り専用） | `database/creds/system-auth-ro` | Vault Database エンジンで自動生成（TTL: 24時間） |

### docker-compose（ローカル開発）

[docker-compose設計](docker-compose設計.md) の共通 PostgreSQL インスタンスに `k1s0_system` データベースを作成する。初期化スクリプトは `infra/docker/init-db/` に配置する。

```sql
-- infra/docker/init-db/01-create-databases.sql（抜粋）
CREATE DATABASE k1s0_system;
```

---

## スキーマ定義ファイル

参照用のスキーマ定義を `regions/system/database/auth-db/schema/` に配置する。マイグレーションファイルが正であり、schema/ は参照・ドキュメント目的で使用する。

```
schema/
├── auth.sql              # 全テーブルの CREATE 文を統合したリファレンス
└── er-diagram.md         # ER 図（テキスト形式）
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

### バックアップ方針

| 項目 | 値 |
|------|-----|
| フルバックアップ | 毎日深夜（0:00） |
| WAL アーカイブ | 継続的（PITR 対応） |
| バックアップ先 | Ceph オブジェクトストレージ |
| 保持期間 | フルバックアップ: 30日、WAL: 7日 |
| リストアテスト | 月次で staging 環境にリストアし検証 |

### バックアップ実行例

```bash
# フルバックアップ（pg_basebackup）
pg_basebackup -h postgres.k1s0-system.svc.cluster.local -U replication -D /backup/base -Ft -z -P

# 論理バックアップ（pg_dump）
pg_dump -h postgres.k1s0-system.svc.cluster.local -U app -d k1s0_system -Fc -f /backup/k1s0_system.dump
```

---

## 関連ドキュメント

- [tier-architecture](tier-architecture.md) — Tier アーキテクチャ・データベースアクセスルール
- [認証認可設計](認証認可設計.md) — OAuth 2.0 / OIDC・RBAC・Vault シークレット管理
- [config設計](config設計.md) — config.yaml スキーマ（database セクション）
- [テンプレート仕様-データベース](テンプレート仕様-データベース.md) — マイグレーション命名規則・テンプレート
- [コンセプト](コンセプト.md) — 技術スタック（PostgreSQL 17・sqlx）
- [ディレクトリ構成図](ディレクトリ構成図.md) — データベース配置先ディレクトリ
- [docker-compose設計](docker-compose設計.md) — ローカル開発用 PostgreSQL
- [可観測性設計](可観測性設計.md) — OpenTelemetry トレース ID 連携
- [kubernetes設計](kubernetes設計.md) — Namespace・PVC 設計
- [helm設計](helm設計.md) — PostgreSQL Helm Chart・Vault Agent Injector
