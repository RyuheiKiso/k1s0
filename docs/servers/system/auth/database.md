# system-auth-server データベース設計

## スキーマ

スキーマ名: `auth`

```sql
CREATE SCHEMA IF NOT EXISTS auth;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| users | Keycloak 連携ユーザー |
| roles | ロール定義（system/business/service Tier） |
| permissions | パーミッション定義（リソース×アクション） |
| user_roles | ユーザーとロールの関連（中間テーブル） |
| role_permissions | ロールとパーミッションの関連（中間テーブル） |
| audit_logs | 監査ログ（月次パーティショニング） |
| api_keys | API キー管理 |

---

## ER 図

```
users 1──* user_roles *──1 roles 1──* role_permissions *──1 permissions
users 1──* api_keys (created_by)
users 1──* user_roles (assigned_by)
audit_logs (パーティションテーブル、FK なし)
```

---

## テーブル定義

### users（ユーザー）

Keycloak の Subject ID と紐付くユーザー情報を管理する。

```sql
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

CREATE INDEX IF NOT EXISTS idx_users_keycloak_sub ON auth.users (keycloak_sub);
CREATE INDEX IF NOT EXISTS idx_users_status ON auth.users (status);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON auth.users (created_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| keycloak_sub | VARCHAR(255) | UNIQUE, NOT NULL | Keycloak Subject ID |
| username | VARCHAR(255) | UNIQUE, NOT NULL | ユーザー名 |
| email | VARCHAR(255) | UNIQUE, NOT NULL | メールアドレス |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'active' | ステータス（active/inactive/suspended） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### roles（ロール定義）

system/business/service の Tier 別ロールを定義する。

```sql
CREATE TABLE IF NOT EXISTS auth.roles (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    tier        VARCHAR(20)  NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 014_add_timestamps_to_roles_and_permissions.up.sql にて追加（ロール変更日時の追跡）
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_roles_tier CHECK (tier IN ('system', 'business', 'service'))
);

CREATE INDEX IF NOT EXISTS idx_roles_tier ON auth.roles (tier);
CREATE INDEX IF NOT EXISTS idx_roles_name ON auth.roles (name);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(100) | UNIQUE, NOT NULL | ロール名 |
| description | TEXT | | 説明 |
| tier | VARCHAR(20) | NOT NULL | Tier（system/business/service） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時（migration 014 で追加） |

---

### permissions（パーミッション定義）

リソースとアクションの組み合わせでパーミッションを定義する。

```sql
CREATE TABLE IF NOT EXISTS auth.permissions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    resource    VARCHAR(255) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    description TEXT,
    -- 014_add_timestamps_to_roles_and_permissions.up.sql にて追加（権限の作成・変更日時の追跡）
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_permissions_resource_action UNIQUE (resource, action),
    CONSTRAINT chk_permissions_action CHECK (action IN ('read', 'write', 'delete', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_permissions_resource ON auth.permissions (resource);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| resource | VARCHAR(255) | UNIQUE(resource, action), NOT NULL | リソース名 |
| action | VARCHAR(50) | UNIQUE(resource, action), NOT NULL | アクション（read/write/delete/admin） |
| description | TEXT | | 説明 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時（migration 014 で追加） |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時（migration 014 で追加） |

---

### user_roles（ユーザー・ロール関連）

ユーザーへのロール割り当てを管理する中間テーブル。

```sql
CREATE TABLE IF NOT EXISTS auth.user_roles (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    role_id     UUID        NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
    assigned_by UUID        REFERENCES auth.users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_user_roles_user_role UNIQUE (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON auth.user_roles (user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON auth.user_roles (role_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| user_id | UUID | FK → users.id, NOT NULL | ユーザー ID |
| role_id | UUID | FK → roles.id, NOT NULL | ロール ID |
| assigned_by | UUID | FK → users.id | 割り当て実行者 |
| assigned_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 割り当て日時 |

---

### role_permissions（ロール・パーミッション関連）

ロールへのパーミッション付与を管理する中間テーブル。

```sql
CREATE TABLE IF NOT EXISTS auth.role_permissions (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id       UUID        NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
    permission_id UUID        NOT NULL REFERENCES auth.permissions(id) ON DELETE CASCADE,
    granted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_role_permissions_role_permission UNIQUE (role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON auth.role_permissions (role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON auth.role_permissions (permission_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| role_id | UUID | FK → roles.id, NOT NULL | ロール ID |
| permission_id | UUID | FK → permissions.id, NOT NULL | パーミッション ID |
| granted_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 付与日時 |

---

### audit_logs（監査ログ）

認証・認可イベントの監査ログ。月次パーティショニングで運用し、pg_partman による自動管理（24 ヶ月保持）を行う。

```sql
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

CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id_created_at
    ON auth.audit_logs (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type_created_at
    ON auth.audit_logs (event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at
    ON auth.audit_logs (action, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_trace_id
    ON auth.audit_logs (trace_id) WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON auth.audit_logs (resource, resource_id) WHERE resource IS NOT NULL;
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK (複合) | 主キー |
| user_id | TEXT | | 操作ユーザー ID |
| event_type | VARCHAR(100) | NOT NULL | イベント種別 |
| action | VARCHAR(100) | NOT NULL | アクション |
| resource | VARCHAR(255) | | 対象リソース |
| resource_id | VARCHAR(255) | | 対象リソース ID |
| result | VARCHAR(50) | NOT NULL, DEFAULT 'SUCCESS' | 結果 |
| detail | JSONB | | 詳細データ |
| ip_address | TEXT | | IP アドレス |
| user_agent | TEXT | | User-Agent |
| trace_id | VARCHAR(64) | | トレース ID |
| created_at | TIMESTAMPTZ | PK (複合), NOT NULL, DEFAULT NOW() | 記録日時（パーティションキー） |

---

### api_keys（API キー管理）

サービス間認証用の API キーを管理する。キー本体はハッシュ化して保存し、プレフィックスで識別する。

```sql
CREATE TABLE IF NOT EXISTS auth.api_keys (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL,
    key_hash     VARCHAR(255) UNIQUE NOT NULL,
    prefix       VARCHAR(10)  NOT NULL,
    tenant_id    VARCHAR(255) NOT NULL,
    tier         VARCHAR(20)  NOT NULL,
    scopes       JSONB        NOT NULL DEFAULT '[]',
    expires_at   TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked      BOOLEAN      NOT NULL DEFAULT false,
    created_by   UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_api_keys_tier CHECK (tier IN ('system', 'business', 'service'))
);

CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_id ON auth.api_keys (tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON auth.api_keys (prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON auth.api_keys (expires_at) WHERE expires_at IS NOT NULL;

-- HIGH-5 監査対応（Migration 017）: テナント横断プレフィックス列挙攻撃を防止するための複合 UNIQUE 制約
ALTER TABLE auth.api_keys
  ADD CONSTRAINT uk_api_keys_tenant_prefix UNIQUE (tenant_id, prefix);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | NOT NULL | キー名 |
| key_hash | VARCHAR(255) | UNIQUE, NOT NULL | キーハッシュ |
| prefix | VARCHAR(10) | NOT NULL | キープレフィックス（識別用） |
| tenant_id | VARCHAR(255) | NOT NULL | テナント ID |
| tier | VARCHAR(20) | NOT NULL | Tier（system/business/service） |
| scopes | JSONB | NOT NULL, DEFAULT '[]' | 許可スコープ一覧 |
| expires_at | TIMESTAMPTZ | | 有効期限 |
| last_used_at | TIMESTAMPTZ | | 最終使用日時 |
| revoked | BOOLEAN | NOT NULL, DEFAULT false | 失効フラグ |
| created_by | UUID | FK → users.id | 作成者 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/auth-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `auth` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_users.up.sql` | users テーブル作成 |
| `002_create_users.down.sql` | テーブル削除 |
| `003_create_roles.up.sql` | roles テーブル作成 |
| `003_create_roles.down.sql` | テーブル削除 |
| `004_create_permissions.up.sql` | permissions テーブル作成 |
| `004_create_permissions.down.sql` | テーブル削除 |
| `005_create_user_roles_and_role_permissions.up.sql` | user_roles・role_permissions テーブル作成 |
| `005_create_user_roles_and_role_permissions.down.sql` | テーブル削除 |
| `006_create_audit_logs.up.sql` | audit_logs パーティションテーブル作成 |
| `006_create_audit_logs.down.sql` | テーブル削除 |
| `007_create_api_keys.up.sql` | api_keys テーブル作成 |
| `007_create_api_keys.down.sql` | テーブル削除 |
| `008_seed_initial_data.up.sql` | デフォルトロール・権限・マッピング投入 |
| `008_seed_initial_data.down.sql` | 初期データ削除 |
| `009_align_audit_log_columns.up.sql` | audit_logs カラム名リネーム |
| `009_align_audit_log_columns.down.sql` | リネーム復元 |
| `010_fix_audit_log_columns.up.sql` | audit_logs カラム名を正規設計に復元 |
| `010_fix_audit_log_columns.down.sql` | リネーム復元 |
| `011_create_partition_management.up.sql` | pg_partman によるパーティション自動管理設定 |
| `011_create_partition_management.down.sql` | パーティション管理設定削除 |
| `012_align_api_keys_columns.up.sql` | api_keys カラム名を Rust コードと整合（service_name→tenant_id, permissions→scopes, key_prefix→prefix, is_active→revoked） |
| `012_align_api_keys_columns.down.sql` | カラム名復元 |
| `013_squash_audit_columns.up.sql` | audit_logs カラム定義の集約・正規化 |
| `013_squash_audit_columns.down.sql` | 集約・正規化の復元 |
| `014_add_timestamps_to_roles_and_permissions.up.sql` | roles に updated_at、permissions に created_at・updated_at を追加（監査要件対応） |
| `014_add_timestamps_to_roles_and_permissions.down.sql` | タイムスタンプカラム削除 |
| `015_add_updated_at_triggers.up.sql` | roles・permissions の updated_at 自動更新トリガーを追加 |
| `015_add_updated_at_triggers.down.sql` | トリガー削除 |
| `016_add_email_verified.up.sql` | users に email_verified カラム追加 |
| `016_add_email_verified.down.sql` | カラム削除 |
| `017_add_api_keys_tenant_prefix_unique.up.sql` | api_keys に tenant_id+prefix 複合 UNIQUE 制約追加（HIGH-5 監査対応） |
| `017_add_api_keys_tenant_prefix_unique.down.sql` | 複合 UNIQUE 制約削除 |
| `018_add_api_keys_rls.up.sql` | api_keys テーブルに RLS（行レベルセキュリティ）追加（H-010 監査対応） |
| `018_add_api_keys_rls.down.sql` | RLS ポリシー・設定削除 |
| `019_extend_prefix_varchar.up.sql` | api_keys.prefix を VARCHAR(10) → VARCHAR(32) に拡張（B-MEDIUM-06 監査対応） |
| `019_extend_prefix_varchar.down.sql` | prefix カラム型を VARCHAR(10) に復元 |
| `020_fix_api_keys_rls_cast.up.sql` | api_keys の RLS ポリシーに `::TEXT` キャストを追加して型不一致を解消 |
| `020_fix_api_keys_rls_cast.down.sql` | RLS ポリシーをキャストなしバージョンに復元 |

### 019_extend_prefix_varchar.up.sql
**目的**: API キープレフィックスのブルートフォース耐性向上のためのカラム長拡張

| 変更内容 | 詳細 |
|---------|------|
| カラム型変更 | `ALTER TABLE auth.api_keys ALTER COLUMN prefix TYPE VARCHAR(32);` |

**B-MEDIUM-06 監査対応**: プレフィックスを 13〜21 文字に延長するため、カラム型を VARCHAR(10) から VARCHAR(32) に拡張する。VARCHAR(10) では実質格納不可だったプレフィックス長を許容し、ブルートフォース耐性を向上させる。

### 020_fix_api_keys_rls_cast.up.sql
**目的**: api_keys テーブルの RLS ポリシーにおける型不一致の修正

| 変更内容 | 詳細 |
|---------|------|
| RLS ポリシー再作成 | `DROP POLICY IF EXISTS tenant_isolation ON auth.api_keys;` |
| キャスト追加 | `CREATE POLICY tenant_isolation ON auth.api_keys USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);` |

**背景**: migration 019 で prefix 長が拡張されたため 020 として適用。`tenant_id`（TEXT 型）と `current_setting()` の戻り値（TEXT 型）を `::TEXT` キャストで明示的に統一し、RLS ポリシーの型不一致エラーを解消する。トランザクション内で実行（BEGIN/COMMIT）。

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION auth.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_update_updated_at
    BEFORE UPDATE ON auth.users
    FOR EACH ROW EXECUTE FUNCTION auth.update_updated_at();

CREATE TRIGGER trigger_api_keys_update_updated_at
    BEFORE UPDATE ON auth.api_keys
    FOR EACH ROW EXECUTE FUNCTION auth.update_updated_at();

-- migration 015 で追加: roles・permissions の updated_at 自動更新トリガー
CREATE TRIGGER set_roles_updated_at
    BEFORE UPDATE ON auth.roles
    FOR EACH ROW EXECUTE FUNCTION auth.set_updated_at();

CREATE TRIGGER set_permissions_updated_at
    BEFORE UPDATE ON auth.permissions
    FOR EACH ROW EXECUTE FUNCTION auth.set_updated_at();
```
