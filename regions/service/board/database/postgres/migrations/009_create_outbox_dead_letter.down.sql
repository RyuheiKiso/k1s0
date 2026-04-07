-- ARCH-HIGH-003 ロールバック: デッドレタキューテーブルを削除する。
SET LOCAL search_path TO board_service, public;
DROP TABLE IF EXISTS outbox_dead_letter;
