-- saga-db: saga_states および saga_step_logs への追加インデックス

-- saga_step_logs: ステップ名での検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_step_name
    ON saga.saga_step_logs (step_name);

-- saga_step_logs: ステータスでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_status
    ON saga.saga_step_logs (status);

-- saga_step_logs: アクションでのフィルタ用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_action
    ON saga.saga_step_logs (action);

-- saga_step_logs: 開始時刻での範囲検索用
CREATE INDEX IF NOT EXISTS idx_saga_step_logs_started_at
    ON saga.saga_step_logs (started_at);
