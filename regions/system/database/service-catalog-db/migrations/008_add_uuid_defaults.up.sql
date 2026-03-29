-- H-011 監査対応: service_catalog テーブルの PRIMARY KEY に DEFAULT gen_random_uuid() を追加する
-- 他の全テーブルと同様に UUID デフォルト値を設定し、INSERT 時の明示的 UUID 指定を不要にする

BEGIN;

-- services テーブルに UUID デフォルトを追加する
ALTER TABLE service_catalog.services ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- teams テーブルに UUID デフォルトを追加する
ALTER TABLE service_catalog.teams ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- service_docs テーブルの存在を確認して UUID デフォルトを追加する（存在する場合のみ）
DO $$
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'service_catalog' AND table_name = 'service_docs'
    ) THEN
        ALTER TABLE service_catalog.service_docs ALTER COLUMN id SET DEFAULT gen_random_uuid();
    END IF;
END $$;

COMMIT;
