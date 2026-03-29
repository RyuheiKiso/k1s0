-- ステータス更新者の監査証跡を記録するため updated_by カラムを追加する（HIGH-07 対応）
SET search_path TO activity_service;

ALTER TABLE activities
    ADD COLUMN IF NOT EXISTS updated_by TEXT;
