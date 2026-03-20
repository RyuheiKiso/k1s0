-- tenant-db: owner_id カラムの型を UUID から VARCHAR(255) に戻す
ALTER TABLE tenant.tenants
    ALTER COLUMN owner_id TYPE VARCHAR(255)
        USING owner_id::TEXT;
