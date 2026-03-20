-- auth-db: roles の updated_at および permissions の created_at / updated_at を削除する
ALTER TABLE auth.roles
    DROP COLUMN IF EXISTS updated_at;

ALTER TABLE auth.permissions
    DROP COLUMN IF EXISTS created_at,
    DROP COLUMN IF EXISTS updated_at;
