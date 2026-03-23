-- updated_by カラムを元の VARCHAR(255) nullable 型に戻すロールバック用マイグレーション
SET search_path TO task_service;

ALTER TABLE tasks ALTER COLUMN updated_by TYPE VARCHAR(255);
ALTER TABLE tasks ALTER COLUMN updated_by DROP NOT NULL;
ALTER TABLE tasks ALTER COLUMN updated_by DROP DEFAULT;
