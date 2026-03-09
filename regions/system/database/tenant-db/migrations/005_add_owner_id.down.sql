-- tenant-db: owner_id カラムの削除

ALTER TABLE tenant.tenants DROP COLUMN IF EXISTS owner_id;
