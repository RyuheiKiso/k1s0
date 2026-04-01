-- STATIC-CRITICAL-002 監査対応ロールバック: cosign_signature カラムを削除する
DROP INDEX IF EXISTS app_registry.idx_app_versions_cosign_signature;
ALTER TABLE app_registry.app_versions
    DROP COLUMN IF EXISTS cosign_signature;
