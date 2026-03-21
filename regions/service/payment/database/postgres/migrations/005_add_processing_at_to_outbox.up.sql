-- outbox_events テーブルにポーラーによるクレーム状態を記録するカラムを追加する
ALTER TABLE outbox_events ADD COLUMN IF NOT EXISTS processing_at TIMESTAMPTZ;
-- スケジュールされた未処理のイベントを効率的に検索するためのインデックス
CREATE INDEX IF NOT EXISTS idx_outbox_events_unprocessed
  ON outbox_events (created_at ASC)
  WHERE published_at IS NULL AND processing_at IS NULL;
