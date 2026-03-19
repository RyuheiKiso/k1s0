CREATE TABLE IF NOT EXISTS app_registry.download_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id TEXT NOT NULL REFERENCES app_registry.apps(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    user_id TEXT NOT NULL,
    downloaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_download_stats_app_id ON app_registry.download_stats(app_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON app_registry.download_stats(downloaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_download_stats_user_id ON app_registry.download_stats(user_id);
