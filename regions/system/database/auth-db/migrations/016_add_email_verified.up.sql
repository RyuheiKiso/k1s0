-- MED-01 監査対応: email_verified カラムを auth.users テーブルに追加する。
-- 既存ユーザーは email_verified = false で初期化し、
-- 検証済みユーザーは管理画面またはマイグレーション後スクリプトで更新すること。
ALTER TABLE auth.users
    ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT false;
