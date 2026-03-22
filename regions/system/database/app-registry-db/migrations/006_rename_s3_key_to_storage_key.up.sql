-- s3_key カラムをストレージ実装に依存しない汎用名 storage_key に改名する。
-- S3/AWS SDK依存を完全に除去し、ローカルFSに移行するための変更。
ALTER TABLE app_registry.app_versions RENAME COLUMN s3_key TO storage_key;
