-- LOW-010 監査対応: Kafka パブリッシュ失敗時のイベント消失を防止するため publish_status カラムを追加する。
-- リトライ上限到達後に 'publish_failed' に設定し、dlq-manager 等の再送ジョブで回収できるようにする。
ALTER TABLE eventstore.events ADD COLUMN IF NOT EXISTS publish_status VARCHAR(20) NOT NULL DEFAULT 'pending';

-- パブリッシュ失敗イベントの再送ジョブが効率よく検索できるようインデックスを追加する。
-- WHERE 句で 'publish_failed' のみを対象とした部分インデックスにして効率化する。
CREATE INDEX IF NOT EXISTS idx_events_publish_status_failed
    ON eventstore.events (stream_id, occurred_at)
    WHERE publish_status = 'publish_failed';

-- 正常パブリッシュ済みイベントを 'published' に一括更新する（既存データのマイグレーション）。
-- DEFAULT の 'pending' では過去イベントが再送対象に混入するため、既存データは 'published' とする。
UPDATE eventstore.events SET publish_status = 'published';
