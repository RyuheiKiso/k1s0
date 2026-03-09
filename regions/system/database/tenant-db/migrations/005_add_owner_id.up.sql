-- tenant-db: owner_id カラムの追加

ALTER TABLE tenant.tenants ADD COLUMN IF NOT EXISTS owner_id VARCHAR(255);
