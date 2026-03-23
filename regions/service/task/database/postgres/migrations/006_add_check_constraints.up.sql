-- status/priority カラムに CHECK 制約を追加してDBレベルの入力検証を強化する
-- 設計根拠: 不正な値がアプリケーション層を通過しても DB で必ず拒否されるようにする（多層防御）
SET search_path TO task_service;

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_status
    CHECK (status IN ('open', 'in_progress', 'done', 'cancelled'));

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_priority
    CHECK (priority IN ('low', 'medium', 'high', 'critical'));
