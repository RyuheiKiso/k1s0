DROP TRIGGER IF EXISTS trigger_secrets_update_updated_at ON vault.secrets;
DROP INDEX IF EXISTS vault.idx_secrets_key_path;
DROP TABLE IF EXISTS vault.secrets;
