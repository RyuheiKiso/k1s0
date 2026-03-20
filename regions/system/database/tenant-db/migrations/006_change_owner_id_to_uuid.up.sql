-- tenant-db: owner_id カラムの型を VARCHAR(255) から UUID に変更する
-- 型安全性を高め、auth.users.id（UUID）との外部参照整合性を保証する

-- 既存データを UUID にキャストして型変換する（不正値は NULL に変換）
ALTER TABLE tenant.tenants
    ALTER COLUMN owner_id TYPE UUID
        USING CASE
            WHEN owner_id ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
            THEN owner_id::UUID
            ELSE NULL
        END;
