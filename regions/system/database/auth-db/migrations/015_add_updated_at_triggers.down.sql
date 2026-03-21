-- auth-db: updated_at トリガーと関数を削除する（ロールバック用）

DROP TRIGGER IF EXISTS set_permissions_updated_at ON auth.permissions;
DROP TRIGGER IF EXISTS set_roles_updated_at ON auth.roles;
DROP FUNCTION IF EXISTS auth.set_updated_at();
