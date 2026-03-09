-- auth-db: api_keys カラム名を Rust コードと整合させる

-- カラム名変更
ALTER TABLE auth.api_keys RENAME COLUMN service_name TO tenant_id;
ALTER TABLE auth.api_keys RENAME COLUMN permissions TO scopes;
ALTER TABLE auth.api_keys RENAME COLUMN key_prefix TO prefix;

-- revoked カラム追加（is_active の反転）
ALTER TABLE auth.api_keys ADD COLUMN revoked BOOLEAN NOT NULL DEFAULT false;
UPDATE auth.api_keys SET revoked = NOT is_active;
ALTER TABLE auth.api_keys DROP COLUMN is_active;

-- 旧インデックス削除
DROP INDEX IF EXISTS auth.idx_api_keys_key_hash;
DROP INDEX IF EXISTS auth.idx_api_keys_service_name;
DROP INDEX IF EXISTS auth.idx_api_keys_key_prefix;

-- 新インデックス作成
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_id ON auth.api_keys (tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON auth.api_keys (prefix);
