-- CRITICAL-RUST-001 監査対応: RLS FORCE が有効な auth.api_keys テーブルに対して
-- テナント ID 不明のまま操作が必要なユースケース（認証ブートストラップ・管理操作）のための
-- SECURITY DEFINER 関数を作成する。
-- SECURITY DEFINER により関数はオーナー（マイグレーション実行ロール = DB オーナー）の
-- 権限で実行され RLS をバイパスできる。
-- 対象: find_by_prefix（認証時テナント不明）/ find_by_id / revoke / delete（管理操作）
BEGIN;

-- API キー認証ブートストラップ用: プレフィックスで検索する。
-- テナント ID 不明のまま認証フローで使用するため RLS バイパスが必要。
CREATE OR REPLACE FUNCTION auth.api_key_find_by_prefix(prefix_in TEXT)
RETURNS TABLE(
    id         UUID,
    tenant_id  TEXT,
    name       TEXT,
    key_hash   TEXT,
    prefix     TEXT,
    scopes     JSONB,
    expires_at TIMESTAMPTZ,
    revoked    BOOL,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
)
LANGUAGE sql
SECURITY DEFINER
STABLE
SET search_path = auth, public
AS $$
    SELECT id, tenant_id, name, key_hash, prefix, scopes,
           expires_at, revoked, created_at, updated_at
    FROM auth.api_keys
    WHERE prefix = prefix_in;
$$;

-- API キー管理: UUID で検索する（テナント横断の管理操作）。
CREATE OR REPLACE FUNCTION auth.api_key_find_by_id(id_in UUID)
RETURNS TABLE(
    id         UUID,
    tenant_id  TEXT,
    name       TEXT,
    key_hash   TEXT,
    prefix     TEXT,
    scopes     JSONB,
    expires_at TIMESTAMPTZ,
    revoked    BOOL,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
)
LANGUAGE sql
SECURITY DEFINER
STABLE
SET search_path = auth, public
AS $$
    SELECT id, tenant_id, name, key_hash, prefix, scopes,
           expires_at, revoked, created_at, updated_at
    FROM auth.api_keys
    WHERE id = id_in;
$$;

-- API キー管理: 失効する（テナント横断の管理操作）。
-- 更新した行の id を返す。0 行の場合はキーが存在しない。
CREATE OR REPLACE FUNCTION auth.api_key_revoke(id_in UUID)
RETURNS TABLE(id UUID)
LANGUAGE sql
SECURITY DEFINER
VOLATILE
SET search_path = auth, public
AS $$
    UPDATE auth.api_keys
    SET revoked = true, updated_at = NOW()
    WHERE id = id_in
    RETURNING id;
$$;

-- API キー管理: 削除する（テナント横断の管理操作）。
-- 削除した行の id を返す。0 行の場合はキーが存在しない。
CREATE OR REPLACE FUNCTION auth.api_key_delete(id_in UUID)
RETURNS TABLE(id UUID)
LANGUAGE sql
SECURITY DEFINER
VOLATILE
SET search_path = auth, public
AS $$
    DELETE FROM auth.api_keys
    WHERE id = id_in
    RETURNING id;
$$;

COMMIT;
