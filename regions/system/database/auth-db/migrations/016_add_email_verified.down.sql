-- MED-01 監査対応ロールバック: email_verified カラムを auth.users テーブルから削除する。
ALTER TABLE auth.users DROP COLUMN IF EXISTS email_verified;
