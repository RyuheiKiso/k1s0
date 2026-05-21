-- k1s0 tier2 migration: 0005_apply_rls_policies_all_tables
-- migration_id: 0005_apply_rls_policies_all_tables
-- description: 0004 で作成した tenant_master / pii_data テーブルに RLS FORCE を適用する
-- safety_level: requires_lock
-- table_class: tenant_master
-- applied_at: 2026-05-17T00:00:00Z
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- ============================================================
-- k1s0app ロールへの新規テーブルアクセス権付与
-- ============================================================
-- k1s0app ロールは 0002 で作成済み。0004 で追加したテーブルへの権限を付与する
-- GRANT は idempotent ではないため DO ブロックで存在確認してから実行する
DO $$
BEGIN
    -- k1s0app ロールが存在する場合のみ GRANT を実行する
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0app') THEN
        -- tenant_master への DML 権限を k1s0app ロールに付与する
        EXECUTE 'GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.tenant_master TO k1s0app';
        -- pii_data への DML 権限を k1s0app ロールに付与する（SELECT は purpose check で制限する）
        EXECUTE 'GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.pii_data TO k1s0app';
        -- platform_config への SELECT 権限を k1s0app ロールに付与する（書込は platform_admin のみ）
        EXECUTE 'GRANT SELECT ON k1s0.platform_config TO k1s0app';
    END IF;
END;
$$;

-- platform_admin ロールを作成する（platform_config 書込専用ロール）
DO $$
BEGIN
    -- platform_admin ロールが存在しない場合のみ作成する
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'platform_admin') THEN
        -- プラットフォーム管理者専用ロールを作成する（NOLOGIN: 直接ログイン禁止 / NOINHERIT: ロール継承禁止）
        -- spec §162「全 role は NOLOGIN NOINHERIT」準拠（Y-tier2-3 修正）
        CREATE ROLE platform_admin NOLOGIN NOINHERIT;
    END IF;
END;
$$;

-- platform_admin ロールに platform_config への全 DML 権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.platform_config TO platform_admin;

-- ============================================================
-- tenant_master テーブルへの RLS FORCE 適用
-- ============================================================
-- table_classes.yaml: tenant_master.rls_enabled = true, rls_policy = tenant_id_match_with_role
-- tenant_id 一致 + role 制限ポリシーを適用する

-- tenant_master テーブルに RLS を有効にする
ALTER TABLE k1s0.tenant_master ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.tenant_master FORCE ROW LEVEL SECURITY;

-- tenant_master のテナント分離 + role check RLS ポリシーを作成する
-- table_classes.yaml: rls_policy = tenant_id_match_with_role（tenant_id + master_admin role 必須）
DROP POLICY IF EXISTS k1s0_tenant_master_tenant_isolation ON k1s0.tenant_master;
CREATE POLICY k1s0_tenant_master_tenant_isolation
    ON k1s0.tenant_master
    -- 全操作（SELECT / INSERT / UPDATE / DELETE）に RESTRICTIVE で適用する
    AS RESTRICTIVE
    -- 既存行の参照制限: tenant_id が GUC と一致し、かつ master_admin ロールのメンバーである場合のみ可視化する
    USING (
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND pg_has_role(current_user, 'master_admin', 'MEMBER')
    )
    -- 新規行の投入制限: tenant_id が GUC と一致し、かつ master_admin ロールのメンバーである場合のみ書込を許可する
    WITH CHECK (
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND pg_has_role(current_user, 'master_admin', 'MEMBER')
    );

-- ============================================================
-- pii_data テーブルへの RLS FORCE + purpose check 適用
-- ============================================================
-- table_classes.yaml: pii_segregated.rls_enabled = true, rls_policy = tenant_id_match_with_purpose_check
-- tenant_id 一致 + app.purpose GUC による目的チェックポリシーを適用する

-- pii_data テーブルに RLS を有効にする
ALTER TABLE k1s0.pii_data ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.pii_data FORCE ROW LEVEL SECURITY;

-- pii_data のテナント分離 + purpose check RLS ポリシーを作成する（SELECT 制限用）
-- 10_テナント分離適合仕様.md §146: pii_segregated の RLS は support / export / emergency のみ許可する
-- 冪等化: DROP + CREATE パターンで idempotent にする（Y-tier2-2 修正）
DROP POLICY IF EXISTS k1s0_pii_data_tenant_isolation_select ON k1s0.pii_data;
CREATE POLICY k1s0_pii_data_tenant_isolation_select
    ON k1s0.pii_data
    -- SELECT 操作のみに適用する（書込ポリシーは別途定義する）
    FOR SELECT
    AS RESTRICTIVE
    -- 参照制限: tenant_id 一致 + purpose check の両方を満たす行のみ可視化する
    USING (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が support / export / emergency のいずれかであることを確認する
        current_setting('app.purpose', true) IN ('support', 'export', 'emergency')
    );

-- pii_data の書込 RLS ポリシーを作成する（INSERT / UPDATE / DELETE 用）
-- 書込も tenant_id 一致 + purpose check を適用する
-- 冪等化: DROP + CREATE パターン（Y-tier2-2 修正）
DROP POLICY IF EXISTS k1s0_pii_data_tenant_isolation_write ON k1s0.pii_data;
CREATE POLICY k1s0_pii_data_tenant_isolation_write
    ON k1s0.pii_data
    -- INSERT / UPDATE / DELETE 操作に適用する
    FOR ALL
    AS RESTRICTIVE
    -- 既存行の参照制限: tenant_id 一致 + purpose check を要求する
    USING (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が support / export / emergency のいずれかであることを確認する
        current_setting('app.purpose', true) IN ('support', 'export', 'emergency')
    )
    -- 新規行の投入制限: tenant_id が GUC と一致する行のみ書込を許可する
    WITH CHECK (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が support / export / emergency のいずれかであることを確認する
        current_setting('app.purpose', true) IN ('support', 'export', 'emergency')
    );

-- ============================================================
-- platform_config への RLS 非適用（platform_global クラス）
-- ============================================================
-- table_classes.yaml: platform_global.rls_enabled = false
-- 全テナントが read 可能なプラットフォームグローバルデータのため RLS を適用しない
-- 書込制限は platform_admin ロールによる GRANT / REVOKE で制御する
-- コメントのみ記載し、RLS 有効化コマンドは実行しない（意図的に RLS を使わないことを明示する）

-- ============================================================
-- 0002 適用済みテーブルへの非侵入確認
-- ============================================================
-- domain_event / outbox_message / audit_event は 0002 で RLS FORCE 適用済みのため
-- 本 migration では操作しない（idempotent のため既存ポリシーへの再適用は不要）
-- ALTER TABLE IF NOT EXISTS は RLS の有効・無効 ALTER には使用できないが
-- 0002 で適用済みであるため問題なし（migration は常に 0002 → 0005 の順で実行される）
