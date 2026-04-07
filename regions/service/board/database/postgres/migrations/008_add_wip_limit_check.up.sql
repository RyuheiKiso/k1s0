-- ARCH-HIGH-005 対応: wip_limit が非負であることを保証する CHECK 制約を追加する。
-- task_count には既に CHECK (task_count >= 0) が設定されているが、
-- wip_limit には制約がなく負の値が挿入可能な状態だった（0 は「制限なし」として扱われる）。
SET LOCAL search_path TO board_service, public;
ALTER TABLE board_columns ADD CONSTRAINT chk_wip_limit CHECK (wip_limit >= 0);
