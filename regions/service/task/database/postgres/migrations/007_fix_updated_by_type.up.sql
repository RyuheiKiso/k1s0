-- updated_by カラムを created_by と同じ TEXT NOT NULL 型に統一する
-- 設計根拠: 型の不統一はアプリケーション層での暗黙的キャスト漏れや予期しないエラーの原因となるため統一する
SET search_path TO task_service;

ALTER TABLE tasks ALTER COLUMN updated_by TYPE TEXT;
ALTER TABLE tasks ALTER COLUMN updated_by SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN updated_by SET DEFAULT '';
