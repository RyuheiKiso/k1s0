-- H-012 監査対応: notification.channels にテナント ID カラムを追加し、マルチテナント分離を実現する
-- H-010 監査対応: RLS（Row Level Security）ポリシーを追加し、DB 層でテナント間データ漏洩を防止する
-- 既存データは DEFAULT 'system' でシステムチャンネルとして扱う

BEGIN;

ALTER TABLE notification.channels
    ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'system';

COMMENT ON COLUMN notification.channels.tenant_id IS
    'テナント識別子。システム共通チャンネルは "system"、テナント固有チャンネルはテナント ID を設定する。';

CREATE INDEX IF NOT EXISTS idx_channels_tenant_id ON notification.channels (tenant_id);

-- RLS を有効化する（FORCE でテーブルオーナーにも適用）
ALTER TABLE notification.channels ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.channels FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシー: app.current_tenant_id セッション変数と一致する行のみアクセス可能
-- set_config('app.current_tenant_id', tenant_id, true) をリポジトリ層で呼び出すことで有効化する
CREATE POLICY tenant_isolation ON notification.channels
    USING (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
