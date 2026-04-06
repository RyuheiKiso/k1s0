-- dlq_messages テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- VARCHAR(255) と TEXT の型不一致により RLS ポリシーのキャストが不要になる。
-- 既存の RLS ポリシーを DROP してから型変更し、AS RESTRICTIVE + WITH CHECK で再作成する。

BEGIN;

SET LOCAL search_path TO dlq, public;

-- 既存の RLS ポリシーを先に削除する（型変更の前提条件）
DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages;

-- tenant_id カラムを VARCHAR(255) から TEXT 型に変更する
ALTER TABLE dlq.dlq_messages
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- テナント分離 RLS ポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
-- TEXT 型になったため ::TEXT キャストが不要になった
CREATE POLICY tenant_isolation ON dlq.dlq_messages
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
