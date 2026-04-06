-- dlq.dlq_messages_archive テーブルにテナント分離を実装する。
-- HIGH-DB-005 監査対応: dlq_messages は 004/005/006 で対応済みだが、
-- LIKE INCLUDING ALL で作成された archive テーブルは tenant_id が存在しない可能性がある。
-- アーカイブデータにもテナント分離を適用してテナント間の漏洩を防止する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO dlq, public;

-- dlq_messages_archive テーブルに tenant_id カラムが存在しない場合のみ追加する
-- （LIKE INCLUDING ALL で元テーブルのカラムが含まれる可能性があるため条件付きで処理）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'dlq'
          AND table_name = 'dlq_messages_archive'
          AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE dlq.dlq_messages_archive
            ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'system';
        ALTER TABLE dlq.dlq_messages_archive
            ALTER COLUMN tenant_id DROP DEFAULT;
    END IF;
END $$;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_dlq_messages_archive_tenant_id
    ON dlq.dlq_messages_archive (tenant_id);

-- dlq_messages_archive テーブルの RLS を有効化する
ALTER TABLE dlq.dlq_messages_archive ENABLE ROW LEVEL SECURITY;
ALTER TABLE dlq.dlq_messages_archive FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON dlq.dlq_messages_archive;
CREATE POLICY tenant_isolation ON dlq.dlq_messages_archive
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
