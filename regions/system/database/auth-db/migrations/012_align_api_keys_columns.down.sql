-- auth-db: api_keys カラム名を元に戻す

-- 旧インデックス削除
DROP INDEX IF EXISTS auth.idx_api_keys_key_hash;
DROP INDEX IF EXISTS auth.idx_api_keys_tenant_id;
DROP INDEX IF EXISTS auth.idx_api_keys_prefix;

-- is_active カラム復元（revoked の反転）
ALTER TABLE auth.api_keys ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;
UPDATE auth.api_keys SET is_active = NOT revoked;
ALTER TABLE auth.api_keys DROP COLUMN revoked;

-- カラム名を元に戻す
ALTER TABLE auth.api_keys RENAME COLUMN tenant_id TO service_name;
ALTER TABLE auth.api_keys RENAME COLUMN scopes TO permissions;
ALTER TABLE auth.api_keys RENAME COLUMN prefix TO key_prefix;

-- 旧インデックス再作成
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_api_keys_service_name ON auth.api_keys (service_name);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON auth.api_keys (key_prefix);
