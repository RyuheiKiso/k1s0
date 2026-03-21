-- tenant-db: owner_id カラムの型を VARCHAR(255) から UUID に変更する
-- 型安全性を高め、auth.users.id（UUID）との外部参照整合性を保証する
--
-- 【運用手順】本マイグレーション実行前に必ず以下の事前確認クエリを実行すること。
-- NULL に変換される（UUID フォーマットに合致しない）レコードが存在する場合は
-- データ修正を先行させること。
--
-- 事前確認クエリ:
--   SELECT id, owner_id
--   FROM tenant.tenants
--   WHERE owner_id IS NOT NULL
--     AND owner_id !~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
--
-- 上記クエリの結果が 0 件であることを確認してから本マイグレーションを実行すること。

-- 既存データを UUID にキャストして型変換する（不正値は NULL に変換）
ALTER TABLE tenant.tenants
    ALTER COLUMN owner_id TYPE UUID
        USING CASE
            WHEN owner_id ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
            THEN owner_id::UUID
            ELSE NULL
        END;
