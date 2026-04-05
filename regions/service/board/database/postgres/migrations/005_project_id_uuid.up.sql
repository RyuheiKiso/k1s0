-- M-007 監査対応: project_id を TEXT から UUID 型に変更してデータ整合性を強化する
-- TEXT 型では不正な値が混入するリスクがあるため、UUID 型で制約を設ける
SET LOCAL search_path TO board_service, public;

-- 既存 project_id カラムを UUID 型へ変更する（USING でキャストする）
ALTER TABLE board_columns
    ALTER COLUMN project_id TYPE UUID
    USING project_id::uuid;
