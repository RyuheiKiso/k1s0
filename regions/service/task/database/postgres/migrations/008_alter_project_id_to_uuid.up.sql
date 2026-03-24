-- project_id カラムを TEXT から UUID 型に変更する
-- 既存データが UUID 形式であることを前提とするため、明示的なキャストを使用する
ALTER TABLE task_service.tasks ALTER COLUMN project_id TYPE UUID USING project_id::uuid;
