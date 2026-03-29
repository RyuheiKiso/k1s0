-- M-014 監査対応: vault.access_policies テーブルに updated_at カラムと自動更新トリガーを追加する
-- 監査カラムの不統一を解消し、ポリシー変更履歴の追跡を可能にする

BEGIN;

-- vault スキーマに updated_at 自動更新関数が存在しない場合は作成する
CREATE OR REPLACE FUNCTION vault.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    -- レコード更新時に updated_at を現在時刻に自動設定する
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- access_policies テーブルに updated_at カラムを追加する
ALTER TABLE vault.access_policies
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- updated_at を自動更新するトリガーを設定する
CREATE TRIGGER trigger_access_policies_update_updated_at
    BEFORE UPDATE ON vault.access_policies
    FOR EACH ROW EXECUTE FUNCTION vault.update_updated_at();

COMMIT;
