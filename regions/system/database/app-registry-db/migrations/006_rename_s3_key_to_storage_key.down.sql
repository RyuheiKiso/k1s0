-- storage_key カラムを元の s3_key に戻すロールバック用マイグレーション。
ALTER TABLE app_registry.app_versions RENAME COLUMN storage_key TO s3_key;
