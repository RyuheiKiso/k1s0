-- ARCH-HIGH-005 ロールバック: wip_limit の CHECK 制約を削除する。
SET LOCAL search_path TO board_service, public;
ALTER TABLE board_columns DROP CONSTRAINT IF EXISTS chk_wip_limit;
