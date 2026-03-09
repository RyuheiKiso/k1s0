-- updated_at 自動更新トリガー関数
CREATE OR REPLACE FUNCTION apiregistry.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- api_schemas テーブルのトリガー
CREATE TRIGGER trg_api_schemas_updated_at
    BEFORE UPDATE ON apiregistry.api_schemas
    FOR EACH ROW
    EXECUTE FUNCTION apiregistry.update_updated_at();
