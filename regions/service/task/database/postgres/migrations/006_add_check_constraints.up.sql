-- status/priority カラムに CHECK 制約を追加してDBレベルの入力検証を強化する
-- 設計根拠: 不正な値がアプリケーション層を通過しても DB で必ず拒否されるようにする（多層防御）
-- 冪等性: init-db が既に制約を作成している場合はスキップする（DO ブロックで重複エラーを無視）
SET search_path TO task_service;

DO $$
BEGIN
    ALTER TABLE tasks
        ADD CONSTRAINT chk_tasks_status
        CHECK (status IN ('open', 'in_progress', 'done', 'cancelled'));
EXCEPTION WHEN duplicate_object THEN
    NULL; -- 既に存在する場合はスキップ
END $$;

DO $$
BEGIN
    ALTER TABLE tasks
        ADD CONSTRAINT chk_tasks_priority
        CHECK (priority IN ('low', 'medium', 'high', 'critical'));
EXCEPTION WHEN duplicate_object THEN
    NULL; -- 既に存在する場合はスキップ
END $$;
