-- auth-db: roles / permissions テーブルの updated_at 自動更新トリガーを追加する
-- 014 マイグレーションで追加した updated_at カラムは DEFAULT NOW() のみで、
-- UPDATE 時に自動更新されない。トリガーで自動更新を保証する。

-- updated_at を現在時刻に更新するトリガー関数（auth スキーマに定義）
CREATE OR REPLACE FUNCTION auth.set_updated_at()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- roles テーブル: UPDATE 時に updated_at を自動更新する
CREATE TRIGGER set_roles_updated_at
    BEFORE UPDATE ON auth.roles
    FOR EACH ROW
    EXECUTE FUNCTION auth.set_updated_at();

-- permissions テーブル: UPDATE 時に updated_at を自動更新する
CREATE TRIGGER set_permissions_updated_at
    BEFORE UPDATE ON auth.permissions
    FOR EACH ROW
    EXECUTE FUNCTION auth.set_updated_at();
