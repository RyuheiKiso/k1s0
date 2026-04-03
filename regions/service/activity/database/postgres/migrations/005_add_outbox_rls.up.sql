-- HIGH-005 監査対応: outbox_events テーブルに tenant_id カラムと RLS ポリシーを追加する。
-- k1s0_service DB は activity/board/task の 3 スキーマを同一 DB に保持しているため、
-- テナント間のイベント参照・操作リスクを RLS で防止する。
-- lessons.md: マイグレーション内では SET LOCAL search_path TO <schema>, public; を使用する。
SET LOCAL search_path TO activity_service, public;

-- 既存の outbox_events レコードには 'system' をデフォルト値として付与する
ALTER TABLE outbox_events
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_outbox_events_tenant_id ON outbox_events (tenant_id);

ALTER TABLE outbox_events ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON outbox_events;
-- バックグラウンドパブリッシャー（set_config 未呼出し）からは全テナントのイベントが参照可能とする。
-- アプリケーションコードが set_config() でテナントを設定した場合は当該テナントのみ参照可能。
CREATE POLICY tenant_isolation ON outbox_events
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
