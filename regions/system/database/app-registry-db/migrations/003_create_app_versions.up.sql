CREATE TABLE IF NOT EXISTS app_registry.app_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id TEXT NOT NULL REFERENCES app_registry.apps(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    arch TEXT NOT NULL,
    size_bytes BIGINT,
    checksum_sha256 TEXT NOT NULL,
    s3_key TEXT NOT NULL,
    release_notes TEXT,
    mandatory BOOLEAN NOT NULL DEFAULT false,
    published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE app_registry.app_versions
    ADD CONSTRAINT uq_app_version_platform_arch
    UNIQUE (app_id, version, platform, arch);

CREATE INDEX IF NOT EXISTS idx_app_versions_app_id ON app_registry.app_versions(app_id);
CREATE INDEX IF NOT EXISTS idx_app_versions_published_at ON app_registry.app_versions(published_at DESC);
