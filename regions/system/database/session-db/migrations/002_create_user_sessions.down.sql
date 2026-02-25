DROP INDEX IF EXISTS session.idx_user_sessions_revoked;
DROP INDEX IF EXISTS session.idx_user_sessions_expires_at;
DROP INDEX IF EXISTS session.idx_user_sessions_user_id;
DROP TABLE IF EXISTS session.user_sessions;
