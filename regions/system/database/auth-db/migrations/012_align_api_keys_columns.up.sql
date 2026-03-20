-- auth-db: api_keys カラム名を Rust コードと整合させる
-- べき等性ガード: 重複実行されても安全に処理する

DO $$ BEGIN
    -- service_name → tenant_id へのリネーム
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'service_name'
    ) THEN
        ALTER TABLE auth.api_keys RENAME COLUMN service_name TO tenant_id;
    END IF;

    -- permissions → scopes へのリネーム
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'permissions'
    ) THEN
        ALTER TABLE auth.api_keys RENAME COLUMN permissions TO scopes;
    END IF;

    -- key_prefix → prefix へのリネーム
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'key_prefix'
    ) THEN
        ALTER TABLE auth.api_keys RENAME COLUMN key_prefix TO prefix;
    END IF;

    -- revoked カラム追加（is_active の反転）
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'revoked'
    ) THEN
        ALTER TABLE auth.api_keys ADD COLUMN revoked BOOLEAN NOT NULL DEFAULT false;

        -- is_active が存在する場合のみデータ移行
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'is_active'
        ) THEN
            UPDATE auth.api_keys SET revoked = NOT is_active;
        END IF;
    END IF;

    -- is_active カラム削除
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'auth' AND table_name = 'api_keys' AND column_name = 'is_active'
    ) THEN
        ALTER TABLE auth.api_keys DROP COLUMN is_active;
    END IF;
END $$;

-- 旧インデックス削除（べき等性あり）
DROP INDEX IF EXISTS auth.idx_api_keys_key_hash;
DROP INDEX IF EXISTS auth.idx_api_keys_service_name;
DROP INDEX IF EXISTS auth.idx_api_keys_key_prefix;

-- 新インデックス作成（べき等性あり）
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON auth.api_keys (key_hash) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_id ON auth.api_keys (tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON auth.api_keys (prefix);
