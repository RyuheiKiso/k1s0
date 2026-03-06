-- ロールバック
DROP TABLE IF EXISTS auth.roles;
DROP TRIGGER IF EXISTS trigger_users_update_updated_at ON auth.users;
DROP TABLE IF EXISTS auth.users;
DROP FUNCTION IF EXISTS auth.update_updated_at();
DROP SCHEMA IF EXISTS auth;
DROP EXTENSION IF EXISTS "pgcrypto";
