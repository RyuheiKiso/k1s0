-- quota-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

-- 拡張機能
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- スキーマ
CREATE SCHEMA IF NOT EXISTS quota;

-- updated_at 自動更新関数
CREATE OR REPLACE FUNCTION quota.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
