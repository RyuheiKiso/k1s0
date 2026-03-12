# system-tenant-server データベース設計

## スキーマ

スキーマ名: `tenant`

```sql
CREATE SCHEMA IF NOT EXISTS tenant;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| tenants | テナント定義 |
| tenant_members | テナントメンバー |

---

## ER 図

```
tenants 1──* tenant_members
```

---

## テーブル定義

### tenants（テナント）

マルチテナントのテナントを管理する。プラン・ステータス・Keycloak realm・DB スキーマの紐付けを持つ。

```sql
CREATE TABLE IF NOT EXISTS tenant.tenants (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL UNIQUE,
    display_name    VARCHAR(255) NOT NULL,
    status          VARCHAR(50)  NOT NULL DEFAULT 'provisioning',
    plan            VARCHAR(50)  NOT NULL DEFAULT 'free',
    settings        JSONB        NOT NULL DEFAULT '{}',
    owner_id        VARCHAR(255),
    keycloak_realm  VARCHAR(255),
    db_schema       VARCHAR(255),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_tenants_status CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted')),
    CONSTRAINT chk_tenants_plan CHECK (plan IN ('free', 'starter', 'professional', 'enterprise'))
);

CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenant.tenants (name);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenant.tenants (status);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE, NOT NULL | テナント名（一意識別子） |
| display_name | VARCHAR(255) | NOT NULL | 表示名 |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'provisioning' | ステータス（provisioning/active/suspended/deleted） |
| plan | VARCHAR(50) | NOT NULL, DEFAULT 'free' | プラン（free/starter/professional/enterprise） |
| settings | JSONB | NOT NULL, DEFAULT '{}' | テナント設定 |
| owner_id | VARCHAR(255) | | テナントオーナーのユーザー ID |
| keycloak_realm | VARCHAR(255) | | Keycloak realm 名 |
| db_schema | VARCHAR(255) | | データベーススキーマ名 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### tenant_members（テナントメンバー）

テナントに所属するメンバーとそのロールを管理する。

```sql
CREATE TABLE IF NOT EXISTS tenant.tenant_members (
    id        UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID         NOT NULL REFERENCES tenant.tenants(id) ON DELETE CASCADE,
    user_id   UUID         NOT NULL,
    role      VARCHAR(50)  NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_members_role CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
    CONSTRAINT uq_tenant_members_tenant_user UNIQUE (tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_tenant_members_tenant_id ON tenant.tenant_members (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_members_user_id ON tenant.tenant_members (user_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| tenant_id | UUID | FK → tenants.id, UNIQUE(tenant_id, user_id), NOT NULL | テナント ID |
| user_id | UUID | UNIQUE(tenant_id, user_id), NOT NULL | ユーザー ID |
| role | VARCHAR(50) | NOT NULL, DEFAULT 'member' | ロール（owner/admin/member/viewer） |
| joined_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 参加日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/tenant-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `tenant` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_tenants.up.sql` | tenants テーブル作成 |
| `002_create_tenants.down.sql` | テーブル削除 |
| `003_create_tenant_members.up.sql` | tenant_members テーブル作成 |
| `003_create_tenant_members.down.sql` | テーブル削除 |
| `004_add_tenant_fields.up.sql` | keycloak_realm・db_schema カラム追加 |
| `004_add_tenant_fields.down.sql` | カラム削除 |
| `005_add_owner_id.up.sql` | owner_id カラム追加 |
| `005_add_owner_id.down.sql` | カラム削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION tenant.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_tenants_update_updated_at
    BEFORE UPDATE ON tenant.tenants
    FOR EACH ROW EXECUTE FUNCTION tenant.update_updated_at();
```
