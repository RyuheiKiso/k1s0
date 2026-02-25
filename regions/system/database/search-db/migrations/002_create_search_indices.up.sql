-- search-db: 検索インデックステーブルの作成

CREATE TABLE IF NOT EXISTS search.search_indices (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL UNIQUE,
    mapping    JSONB        NOT NULL DEFAULT '{}',
    doc_count  BIGINT       NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_indices_name ON search.search_indices (name);

CREATE TRIGGER trigger_search_indices_update_updated_at
    BEFORE UPDATE ON search.search_indices
    FOR EACH ROW
    EXECUTE FUNCTION search.update_updated_at();
