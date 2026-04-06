-- CRITICAL-RUST-001 監査対応: FORCE RLS が有効な tenant.tenants テーブルに対して
-- テナント ID 不明のまま操作が必要なユースケース（認証ブートストラップ・管理操作）のための
-- SECURITY DEFINER 関数を作成する。
-- SECURITY DEFINER により関数はオーナー（マイグレーション実行ロール = DB オーナー）の
-- 権限で実行され RLS をバイパスできる。
-- 対象: find_by_name（認証時テナント名でのルックアップ）/ list_all（管理操作）
BEGIN;

-- テナント認証ブートストラップ用: テナント名で検索する。
-- ログイン時にテナント名から UUID・Keycloak レルムを特定するために必要。
-- テナント ID 不明の状態での呼び出しが必須なため RLS バイパスが必要。
CREATE OR REPLACE FUNCTION tenant.tenant_find_by_name(name_in TEXT)
RETURNS TABLE(
    id              UUID,
    name            TEXT,
    display_name    TEXT,
    status          TEXT,
    plan            TEXT,
    owner_id        TEXT,
    settings        JSONB,
    keycloak_realm  TEXT,
    db_schema       TEXT,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
)
LANGUAGE sql
SECURITY DEFINER
STABLE
SET search_path = tenant, public
AS $$
    SELECT id, name, display_name, status, plan, owner_id,
           settings, keycloak_realm, db_schema, created_at, updated_at
    FROM tenant.tenants
    WHERE name = name_in;
$$;

-- テナント管理操作: 全テナントを一覧取得する（管理者 API）。
-- テナント横断のリスト操作は RLS バイパスが必要。
CREATE OR REPLACE FUNCTION tenant.tenant_list_all(limit_in BIGINT, offset_in BIGINT)
RETURNS TABLE(
    id              UUID,
    name            TEXT,
    display_name    TEXT,
    status          TEXT,
    plan            TEXT,
    owner_id        TEXT,
    settings        JSONB,
    keycloak_realm  TEXT,
    db_schema       TEXT,
    created_at      TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ
)
LANGUAGE sql
SECURITY DEFINER
STABLE
SET search_path = tenant, public
AS $$
    SELECT id, name, display_name, status, plan, owner_id,
           settings, keycloak_realm, db_schema, created_at, updated_at
    FROM tenant.tenants
    ORDER BY created_at DESC
    LIMIT limit_in OFFSET offset_in;
$$;

-- テナント管理操作: 全テナント数を取得する（管理者 API ページネーション用）。
CREATE OR REPLACE FUNCTION tenant.tenant_count_all()
RETURNS BIGINT
LANGUAGE sql
SECURITY DEFINER
STABLE
SET search_path = tenant, public
AS $$
    SELECT COUNT(*) FROM tenant.tenants;
$$;

COMMIT;
