-- マルチテナント対応: セッションにテナントIDを追加する。
-- テナント間でのセッション衝突を防ぐためのカラム。
ALTER TABLE session.user_sessions
  ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

CREATE INDEX IF NOT EXISTS idx_user_sessions_tenant_id ON session.user_sessions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_tenant ON session.user_sessions(user_id, tenant_id);
