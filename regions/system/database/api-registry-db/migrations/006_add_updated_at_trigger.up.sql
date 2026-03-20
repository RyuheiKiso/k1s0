-- updated_at 自動更新トリガー関数（べき等性あり: CREATE OR REPLACE）
CREATE OR REPLACE FUNCTION apiregistry.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- べき等性ガード: トリガーが既に存在する場合は削除してから再作成する
DROP TRIGGER IF EXISTS trg_api_schemas_updated_at ON apiregistry.api_schemas;

-- api_schemas テーブルのトリガー
CREATE TRIGGER trg_api_schemas_updated_at
    BEFORE UPDATE ON apiregistry.api_schemas
    FOR EACH ROW
    EXECUTE FUNCTION apiregistry.update_updated_at();
