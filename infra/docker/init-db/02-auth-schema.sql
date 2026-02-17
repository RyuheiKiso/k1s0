-- infra/docker/init-db/02-auth-schema.sql
-- auth-db マイグレーション結合（regions/system/database/auth-db/migrations/ の全 .up.sql）

\c auth_db;

-- 001: スキーマ・拡張機能・共通関数
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS auth;

CREATE OR REPLACE FUNCTION auth.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 002: users テーブル
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

CREATE TRIGGER trigger_users_update_updated_at
    BEFORE UPDATE ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at();

-- 003: roles テーブル
CREATE TABLE IF NOT EXISTS auth.roles (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    tier        VARCHAR(20)  NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_roles_tier CHECK (tier IN ('system', 'business', 'service'))
);

CREATE INDEX IF NOT EXISTS idx_roles_tier ON auth.roles (tier);
CREATE INDEX IF NOT EXISTS idx_roles_name ON auth.roles (name);

-- 004: permissions テーブル
CREATE TABLE IF NOT EXISTS auth.permissions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    resource    VARCHAR(255) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    description TEXT,

    CONSTRAINT uq_permissions_resource_action UNIQUE (resource, action),
    CONSTRAINT chk_permissions_action CHECK (action IN ('read', 'write', 'delete', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_permissions_resource ON auth.permissions (resource);

-- 005: user_roles / role_permissions 中間テーブル
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

CREATE TABLE IF NOT EXISTS auth.role_permissions (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id       UUID        NOT NULL REFERENCES auth.roles(id) ON DELETE CASCADE,
    permission_id UUID        NOT NULL REFERENCES auth.permissions(id) ON DELETE CASCADE,
    granted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_role_permissions_role_permission UNIQUE (role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON auth.role_permissions (role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON auth.role_permissions (permission_id);

-- 006: audit_logs テーブル（月次パーティショニング）
CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    event_type  VARCHAR(100) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    resource_id VARCHAR(255),
    result      VARCHAR(50)  NOT NULL DEFAULT 'SUCCESS',
    detail      JSONB,
    ip_address  INET,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);

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

CREATE TABLE IF NOT EXISTS auth.audit_logs_default PARTITION OF auth.audit_logs DEFAULT;

-- 007: api_keys テーブル
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

CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_service_name ON auth.api_keys (service_name);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON auth.api_keys (key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON auth.api_keys (expires_at) WHERE expires_at IS NOT NULL;

CREATE TRIGGER trigger_api_keys_update_updated_at
    BEFORE UPDATE ON auth.api_keys
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at();

-- 008: 初期データ投入
INSERT INTO auth.roles (name, description, tier) VALUES
    ('sys_admin',    'システム全体の管理者。すべてのリソースに対する全権限',         'system'),
    ('sys_operator', 'システム運用担当。監視・ログ閲覧・設定変更',                   'system'),
    ('sys_auditor',  '監査担当。全リソースの読み取り専用',                            'system')
ON CONFLICT (name) DO NOTHING;

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

INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

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

INSERT INTO auth.role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM auth.roles r
CROSS JOIN auth.permissions p
WHERE r.name = 'sys_auditor'
  AND p.action = 'read'
  AND p.resource != 'vault_secrets'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 009: カラム名変更
ALTER TABLE auth.audit_logs RENAME COLUMN detail TO metadata;
ALTER TABLE auth.audit_logs RENAME COLUMN created_at TO recorded_at;
