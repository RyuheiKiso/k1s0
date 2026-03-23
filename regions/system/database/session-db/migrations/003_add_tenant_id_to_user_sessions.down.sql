-- マルチテナント対応カラムのロールバック: tenant_id カラムを削除する。
ALTER TABLE session.user_sessions DROP COLUMN IF EXISTS tenant_id;
