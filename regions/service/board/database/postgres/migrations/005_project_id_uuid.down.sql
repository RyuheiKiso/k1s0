-- M-007 監査対応ロールバック: project_id を UUID から TEXT 型に戻す
SET LOCAL search_path TO board_service, public;

-- UUID 型から TEXT 型へロールバックする
ALTER TABLE board_columns
    ALTER COLUMN project_id TYPE TEXT
    USING project_id::text;
