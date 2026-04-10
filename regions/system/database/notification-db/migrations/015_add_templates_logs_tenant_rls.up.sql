-- notification.templates と notification.notification_logs にテナント分離を実装する。
-- HIGH-DB-001 監査対応: channels テーブルには 012/014 で対応済みだが、templates と notification_logs は未対応。
-- マルチテナント環境でテナント間のテンプレート・通知ログ参照を防止する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO notification, public;

-- templates テーブルに tenant_id カラムを追加する
ALTER TABLE notification.templates
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE notification.templates
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_templates_tenant_id
    ON notification.templates (tenant_id);

-- templates テーブルの RLS を有効化する
ALTER TABLE notification.templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.templates FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON notification.templates;
CREATE POLICY tenant_isolation ON notification.templates
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- notification_logs テーブルに tenant_id カラムを追加する
ALTER TABLE notification.notification_logs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE notification.notification_logs
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_notification_logs_tenant_id
    ON notification.notification_logs (tenant_id);

-- notification_logs テーブルの RLS を有効化する
ALTER TABLE notification.notification_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.notification_logs FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON notification.notification_logs;
CREATE POLICY tenant_isolation ON notification.notification_logs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
