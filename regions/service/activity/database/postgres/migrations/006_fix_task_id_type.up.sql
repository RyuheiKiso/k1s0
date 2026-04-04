-- H-008 監査対応: task_id カラムを TEXT から UUID 型に変更する
-- 問題: task サービスの tasks.id は UUID だが activity の task_id は TEXT で型不整合
-- 既存データの移行: valid UUID 文字列は::uuid キャストで変換、不正な値は NULL に変換する
-- lessons.md: マイグレーション内では SET LOCAL search_path TO <schema>, public; を使用する
SET LOCAL search_path TO activity_service, public;

ALTER TABLE activities
    ALTER COLUMN task_id TYPE UUID USING (
        CASE
            WHEN task_id ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
            THEN task_id::uuid
            ELSE NULL
        END
    );
