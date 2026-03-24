-- project_id カラムを UUID から TEXT 型に戻す
ALTER TABLE task_service.tasks ALTER COLUMN project_id TYPE TEXT USING project_id::text;
