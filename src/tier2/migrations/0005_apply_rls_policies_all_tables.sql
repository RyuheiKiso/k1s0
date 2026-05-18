-- k1s0 tier2 migration: 0005_apply_rls_policies_all_tables
-- 0004 で作成した tenant_master / pii_data テーブルに RLS FORCE を適用する
-- 10_テナント分離適合仕様.md の RLS FORCE 要件を物理化する
-- 0002 で適用済みの domain_event / outbox_message / audit_event には触らない
--
-- 実行方法:
--   sqlx migrate run --database-url $DATABASE_URL --source src/tier2/migrations/

-- ============================================================
-- k1s0_app ロールへの新規テーブルアクセス権付与
-- ============================================================
-- k1s0_app ロールは 0002 で作成済み。0004 で追加したテーブルへの権限を付与する
-- GRANT は idempotent ではないため DO ブロックで存在確認してから実行する
DO $$
BEGIN
    -- k1s0_app ロールが存在する場合のみ GRANT を実行する
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0_app') THEN
        -- tenant_master への DML 権限を k1s0_app ロールに付与する
        EXECUTE 'GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.tenant_master TO k1s0_app';
        -- pii_data への DML 権限を k1s0_app ロールに付与する（SELECT は purpose check で制限する）
        EXECUTE 'GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.pii_data TO k1s0_app';
        -- platform_config への SELECT 権限を k1s0_app ロールに付与する（書込は platform_admin のみ）
        EXECUTE 'GRANT SELECT ON k1s0.platform_config TO k1s0_app';
    END IF;
END;
$$;

-- platform_admin ロールを作成する（platform_config 書込専用ロール）
DO $$
BEGIN
    -- platform_admin ロールが存在しない場合のみ作成する
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'k1s0_platform_admin') THEN
        -- プラットフォーム管理者専用ロールを作成する（ログインは許可しない）
        CREATE ROLE k1s0_platform_admin NOLOGIN;
    END IF;
END;
$$;

-- platform_admin ロールに platform_config への全 DML 権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.platform_config TO k1s0_platform_admin;

-- ============================================================
-- tenant_master テーブルへの RLS FORCE 適用
-- ============================================================
-- table_classes.yaml: tenant_master.rls_enabled = true, rls_policy = tenant_id_match_with_role
-- tenant_id 一致 + role 制限ポリシーを適用する

-- tenant_master テーブルに RLS を有効にする
ALTER TABLE k1s0.tenant_master ENABLE ROW LEVEL SECURITY;
-- FORCE: スーパーユーザー以外は全て RLS ポリシーを適用する（バイパス禁止）
ALTER TABLE k1s0.tenant_master FORCE ROW LEVEL SECURITY;

-- tenant_master のテナント分離 RLS ポリシーを作成する
-- app.tenant_id GUC と tenant_master.tenant_id が一致する行のみアクセスを許可する
-- CREATE POLICY は IF NOT EXISTS が PostgreSQL 9.5 以降で使用可能
CREATE POLICY IF NOT EXISTS k1s0_tenant_master_tenant_isolation
    ON k1s0.tenant_master
    -- 全操作（SELECT / INSERT / UPDATE / DELETE）に RESTRICTIVE で適用する
    AS RESTRICTIVE
    -- 既存行の参照制限: tenant_id が GUC と一致する行のみ可視化する
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    -- 新規行の投入制限: tenant_id が GUC と一致する行のみ書込を許可する
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

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
-- PII アクセスは app.purpose が 'pii_access' または 'audit_emit' である場合のみ許可する
CREATE POLICY IF NOT EXISTS k1s0_pii_data_tenant_isolation_select
    ON k1s0.pii_data
    -- SELECT 操作のみに適用する（書込ポリシーは別途定義する）
    FOR SELECT
    AS RESTRICTIVE
    -- 参照制限: tenant_id 一致 + purpose check の両方を満たす行のみ可視化する
    USING (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が pii_access または audit_emit であることを確認する
        current_setting('app.purpose', true) IN ('pii_access', 'audit_emit')
    );

-- pii_data の書込 RLS ポリシーを作成する（INSERT / UPDATE / DELETE 用）
-- 書込も tenant_id 一致 + purpose check を適用する
CREATE POLICY IF NOT EXISTS k1s0_pii_data_tenant_isolation_write
    ON k1s0.pii_data
    -- INSERT / UPDATE / DELETE 操作に適用する
    FOR ALL
    AS RESTRICTIVE
    -- 既存行の参照制限: tenant_id 一致 + purpose check を要求する
    USING (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が pii_access または audit_emit であることを確認する
        current_setting('app.purpose', true) IN ('pii_access', 'audit_emit')
    )
    -- 新規行の投入制限: tenant_id が GUC と一致する行のみ書込を許可する
    WITH CHECK (
        -- テナント ID が GUC の値と一致することを確認する
        tenant_id = current_setting('app.tenant_id', true)::uuid
        AND
        -- セッション目的が pii_access または audit_emit であることを確認する
        current_setting('app.purpose', true) IN ('pii_access', 'audit_emit')
    );

-- ============================================================
-- platform_config への RLS 非適用（platform_global クラス）
-- ============================================================
-- table_classes.yaml: platform_global.rls_enabled = false
-- 全テナントが read 可能なプラットフォームグローバルデータのため RLS を適用しない
-- 書込制限は k1s0_platform_admin ロールによる GRANT / REVOKE で制御する
-- コメントのみ記載し、RLS 有効化コマンドは実行しない（意図的に RLS を使わないことを明示する）

-- ============================================================
-- 0002 適用済みテーブルへの非侵入確認
-- ============================================================
-- domain_event / outbox_message / audit_event は 0002 で RLS FORCE 適用済みのため
-- 本 migration では操作しない（idempotent のため既存ポリシーへの再適用は不要）
-- ALTER TABLE IF NOT EXISTS は RLS の有効・無効 ALTER には使用できないが
-- 0002 で適用済みであるため問題なし（migration は常に 0002 → 0005 の順で実行される）
