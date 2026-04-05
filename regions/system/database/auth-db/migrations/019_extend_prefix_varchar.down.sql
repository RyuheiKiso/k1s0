-- B-MEDIUM-06 監査対応のロールバック: prefix カラムを元の VARCHAR(10) に戻す
-- 注意: 既存データが 10 文字を超える場合、このロールバックはエラーになる可能性がある
ALTER TABLE auth.api_keys
    ALTER COLUMN prefix TYPE VARCHAR(10);
