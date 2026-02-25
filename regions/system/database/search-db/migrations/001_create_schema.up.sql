-- search-db: スキーマ・拡張機能・共通関数の作成 (PostgreSQL 17)

-- 拡張機能
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- スキーマ
CREATE SCHEMA IF NOT EXISTS search;

-- updated_at 自動更新関数
CREATE OR REPLACE FUNCTION search.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
