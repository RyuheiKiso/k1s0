-- search-db: 検索ドキュメントテーブルの作成

CREATE TABLE IF NOT EXISTS search.search_documents (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    index_id      UUID         NOT NULL REFERENCES search.search_indices(id) ON DELETE CASCADE,
    document_id   VARCHAR(255) NOT NULL,
    content       JSONB        NOT NULL DEFAULT '{}',
    search_vector TSVECTOR,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_search_documents_index_doc UNIQUE (index_id, document_id)
);

CREATE INDEX IF NOT EXISTS idx_search_documents_index_id ON search.search_documents (index_id);
CREATE INDEX IF NOT EXISTS idx_search_documents_search_vector ON search.search_documents USING GIN (search_vector);
CREATE INDEX IF NOT EXISTS idx_search_documents_content ON search.search_documents USING GIN (content jsonb_path_ops);

-- search_vector 自動生成トリガー関数
CREATE OR REPLACE FUNCTION search.update_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('simple', COALESCE(NEW.content::text, ''));
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_search_documents_update_vector
    BEFORE INSERT OR UPDATE ON search.search_documents
    FOR EACH ROW
    EXECUTE FUNCTION search.update_search_vector();
