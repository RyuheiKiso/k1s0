-- B-MEDIUM-06 監査対応: API キープレフィックス長の拡張
-- プレフィックスを 13 文字（VARCHAR(10) では実質格納不可）から 21 文字に延長するため、
-- カラム型を VARCHAR(32) に拡張してブルートフォース耐性を向上させる
ALTER TABLE auth.api_keys
    ALTER COLUMN prefix TYPE VARCHAR(32);
