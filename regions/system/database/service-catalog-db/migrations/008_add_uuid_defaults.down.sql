-- 008_add_uuid_defaults のロールバック: UUID デフォルト値を削除する
BEGIN;
ALTER TABLE service_catalog.services ALTER COLUMN id DROP DEFAULT;
ALTER TABLE service_catalog.teams ALTER COLUMN id DROP DEFAULT;
DO $$
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'service_catalog' AND table_name = 'service_docs'
    ) THEN
        ALTER TABLE service_catalog.service_docs ALTER COLUMN id DROP DEFAULT;
    END IF;
END $$;
COMMIT;
