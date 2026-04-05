-- H-008 監査対応: task_id カラムを UUID から TEXT 型に戻す（ロールバック用）
-- UUID 値は TEXT にキャストして文字列表現で保持する
SET LOCAL search_path TO activity_service, public;

ALTER TABLE activities
    ALTER COLUMN task_id TYPE TEXT USING task_id::text;
