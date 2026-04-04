-- C-005 監査対応: task ステータス CHECK 制約に 'review' を追加する
-- 問題: domains entity は Open→InProgress→Review→Done/Cancelled の5状態を定義しているが
-- 006_add_check_constraints で追加された CHECK 制約は 'review' を含まないため
-- review への遷移が DB 制約違反で 500 エラーになる
-- 修正: 制約を DROP して 'review' を含む5値の制約を再作成する
-- 設計根拠: docs/architecture/adr/0083-task-status-transition-enforcement.md 参照
SET LOCAL search_path TO task_service, public;

DO $$
BEGIN
    -- 既存の制約を DROP（存在しない場合はスキップ）
    ALTER TABLE tasks DROP CONSTRAINT IF EXISTS chk_tasks_status;

    -- 5状態（review を含む）の制約を再作成
    ALTER TABLE tasks
        ADD CONSTRAINT chk_tasks_status
        CHECK (status IN ('open', 'in_progress', 'review', 'done', 'cancelled'));
EXCEPTION WHEN duplicate_object THEN
    NULL; -- 既に存在する場合はスキップ
END $$;
