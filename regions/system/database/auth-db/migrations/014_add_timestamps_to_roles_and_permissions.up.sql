-- auth-db: roles テーブルに updated_at カラムを追加する
-- 監査要件に対応し、ロールの変更日時を追跡可能にする
ALTER TABLE auth.roles
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- auth-db: permissions テーブルに created_at / updated_at カラムを追加する
-- 監査要件に対応し、権限の作成・変更日時を追跡可能にする
ALTER TABLE auth.permissions
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
