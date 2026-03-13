-- file_metadata: ファイルメタデータ管理
CREATE TABLE IF NOT EXISTS file_storage.file_metadata (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    filename    VARCHAR(1024) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size_bytes  BIGINT      NOT NULL,
    storage_path VARCHAR(2048) NOT NULL,
    checksum    VARCHAR(128),
    tags        JSONB       NOT NULL DEFAULT '{}',
    metadata    JSONB       NOT NULL DEFAULT '{}',
    status      VARCHAR(50) NOT NULL DEFAULT 'active',
    uploaded_by UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_file_metadata_status CHECK (status IN ('active', 'archived', 'deleted')),
    CONSTRAINT chk_file_metadata_size CHECK (size_bytes >= 0)
);

CREATE INDEX idx_file_metadata_filename ON file_storage.file_metadata (filename);
CREATE INDEX idx_file_metadata_content_type ON file_storage.file_metadata (content_type);
CREATE INDEX idx_file_metadata_status ON file_storage.file_metadata (status);
CREATE INDEX idx_file_metadata_uploaded_by ON file_storage.file_metadata (uploaded_by);
CREATE INDEX idx_file_metadata_created_at ON file_storage.file_metadata (created_at);

CREATE TRIGGER trigger_file_metadata_updated_at
    BEFORE UPDATE ON file_storage.file_metadata
    FOR EACH ROW
    EXECUTE FUNCTION file_storage.update_updated_at();
