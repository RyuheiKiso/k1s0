-- updated_by カラムを削除する（004_add_updated_by.up.sql のロールバック）
SET search_path TO activity_service;

ALTER TABLE activities
    DROP COLUMN IF EXISTS updated_by;
