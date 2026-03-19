CREATE TABLE IF NOT EXISTS app_registry.apps (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    icon_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER apps_updated_at
    BEFORE UPDATE ON app_registry.apps
    FOR EACH ROW
    EXECUTE FUNCTION app_registry.update_updated_at();
