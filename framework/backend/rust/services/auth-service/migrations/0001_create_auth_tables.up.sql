-- auth-service: 認証・認可関連テーブル
-- 所有テーブル: fw_m_user, fw_m_role, fw_m_permission, fw_m_user_role, fw_m_role_permission

-- ユーザーマスタ
CREATE TABLE IF NOT EXISTS fw_m_user (
    user_id BIGSERIAL PRIMARY KEY,
    login_id VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,  -- 0: inactive, 1: active, 2: locked
    failed_login_count SMALLINT NOT NULL DEFAULT 0,
    last_login_at TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT
);

-- ユーザーインデックス
CREATE INDEX idx_fw_m_user_login_id ON fw_m_user(login_id);
CREATE INDEX idx_fw_m_user_email ON fw_m_user(email);
CREATE INDEX idx_fw_m_user_status ON fw_m_user(status);

-- ユーザーコメント
COMMENT ON TABLE fw_m_user IS 'ユーザーマスタ';
COMMENT ON COLUMN fw_m_user.user_id IS 'ユーザーID';
COMMENT ON COLUMN fw_m_user.login_id IS 'ログインID';
COMMENT ON COLUMN fw_m_user.email IS 'メールアドレス';
COMMENT ON COLUMN fw_m_user.display_name IS '表示名';
COMMENT ON COLUMN fw_m_user.password_hash IS 'パスワードハッシュ';
COMMENT ON COLUMN fw_m_user.status IS 'ステータス (0:無効, 1:有効, 2:ロック)';
COMMENT ON COLUMN fw_m_user.failed_login_count IS 'ログイン失敗回数';
COMMENT ON COLUMN fw_m_user.last_login_at IS '最終ログイン日時';
COMMENT ON COLUMN fw_m_user.password_changed_at IS 'パスワード変更日時';

-- ロールマスタ
CREATE TABLE IF NOT EXISTS fw_m_role (
    role_id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,  -- システムロール（削除不可）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT
);

-- ロールインデックス
CREATE INDEX idx_fw_m_role_name ON fw_m_role(role_name);

-- ロールコメント
COMMENT ON TABLE fw_m_role IS 'ロールマスタ';
COMMENT ON COLUMN fw_m_role.role_id IS 'ロールID';
COMMENT ON COLUMN fw_m_role.role_name IS 'ロール名';
COMMENT ON COLUMN fw_m_role.description IS '説明';
COMMENT ON COLUMN fw_m_role.is_system IS 'システムロールフラグ';

-- パーミッションマスタ
CREATE TABLE IF NOT EXISTS fw_m_permission (
    permission_id BIGSERIAL PRIMARY KEY,
    permission_key VARCHAR(255) NOT NULL UNIQUE,  -- 例: "user:read", "order:write"
    resource_type VARCHAR(100) NOT NULL,          -- 例: "user", "order"
    operation VARCHAR(50) NOT NULL,               -- 例: "read", "write", "delete"
    description TEXT,
    service_name VARCHAR(100),                    -- サービススコープ（NULLは全サービス）
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT
);

-- パーミッションインデックス
CREATE INDEX idx_fw_m_permission_key ON fw_m_permission(permission_key);
CREATE INDEX idx_fw_m_permission_resource ON fw_m_permission(resource_type);
CREATE INDEX idx_fw_m_permission_service ON fw_m_permission(service_name);

-- パーミッションコメント
COMMENT ON TABLE fw_m_permission IS 'パーミッションマスタ';
COMMENT ON COLUMN fw_m_permission.permission_id IS 'パーミッションID';
COMMENT ON COLUMN fw_m_permission.permission_key IS 'パーミッションキー';
COMMENT ON COLUMN fw_m_permission.resource_type IS 'リソースタイプ';
COMMENT ON COLUMN fw_m_permission.operation IS '操作';
COMMENT ON COLUMN fw_m_permission.service_name IS 'サービス名スコープ';

-- ユーザー・ロール関連
CREATE TABLE IF NOT EXISTS fw_m_user_role (
    user_id BIGINT NOT NULL REFERENCES fw_m_user(user_id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL REFERENCES fw_m_role(role_id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    granted_by BIGINT,
    expires_at TIMESTAMPTZ,  -- NULLは無期限
    PRIMARY KEY (user_id, role_id)
);

-- ユーザー・ロールインデックス
CREATE INDEX idx_fw_m_user_role_user ON fw_m_user_role(user_id);
CREATE INDEX idx_fw_m_user_role_role ON fw_m_user_role(role_id);

-- ユーザー・ロールコメント
COMMENT ON TABLE fw_m_user_role IS 'ユーザー・ロール関連';
COMMENT ON COLUMN fw_m_user_role.user_id IS 'ユーザーID';
COMMENT ON COLUMN fw_m_user_role.role_id IS 'ロールID';
COMMENT ON COLUMN fw_m_user_role.granted_at IS '付与日時';
COMMENT ON COLUMN fw_m_user_role.granted_by IS '付与者';
COMMENT ON COLUMN fw_m_user_role.expires_at IS '有効期限';

-- ロール・パーミッション関連
CREATE TABLE IF NOT EXISTS fw_m_role_permission (
    role_id BIGINT NOT NULL REFERENCES fw_m_role(role_id) ON DELETE CASCADE,
    permission_id BIGINT NOT NULL REFERENCES fw_m_permission(permission_id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    granted_by BIGINT,
    PRIMARY KEY (role_id, permission_id)
);

-- ロール・パーミッションインデックス
CREATE INDEX idx_fw_m_role_permission_role ON fw_m_role_permission(role_id);
CREATE INDEX idx_fw_m_role_permission_perm ON fw_m_role_permission(permission_id);

-- ロール・パーミッションコメント
COMMENT ON TABLE fw_m_role_permission IS 'ロール・パーミッション関連';
COMMENT ON COLUMN fw_m_role_permission.role_id IS 'ロールID';
COMMENT ON COLUMN fw_m_role_permission.permission_id IS 'パーミッションID';
COMMENT ON COLUMN fw_m_role_permission.granted_at IS '付与日時';
COMMENT ON COLUMN fw_m_role_permission.granted_by IS '付与者';

-- 更新日時トリガー関数
CREATE OR REPLACE FUNCTION fw_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 更新日時トリガー
CREATE TRIGGER tr_fw_m_user_updated
    BEFORE UPDATE ON fw_m_user
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

CREATE TRIGGER tr_fw_m_role_updated
    BEFORE UPDATE ON fw_m_role
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

CREATE TRIGGER tr_fw_m_permission_updated
    BEFORE UPDATE ON fw_m_permission
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

-- 初期データ: システムロール
INSERT INTO fw_m_role (role_name, description, is_system) VALUES
    ('admin', 'システム管理者', TRUE),
    ('user', '一般ユーザー', TRUE),
    ('viewer', '閲覧専用ユーザー', TRUE)
ON CONFLICT (role_name) DO NOTHING;

-- 初期データ: 基本パーミッション
INSERT INTO fw_m_permission (permission_key, resource_type, operation, description) VALUES
    ('user:read', 'user', 'read', 'ユーザー情報の読み取り'),
    ('user:write', 'user', 'write', 'ユーザー情報の作成・更新'),
    ('user:delete', 'user', 'delete', 'ユーザーの削除'),
    ('role:read', 'role', 'read', 'ロール情報の読み取り'),
    ('role:write', 'role', 'write', 'ロールの作成・更新'),
    ('role:delete', 'role', 'delete', 'ロールの削除'),
    ('permission:read', 'permission', 'read', 'パーミッション情報の読み取り'),
    ('permission:write', 'permission', 'write', 'パーミッションの作成・更新'),
    ('admin:all', 'admin', 'all', '管理者権限（すべての操作）')
ON CONFLICT (permission_key) DO NOTHING;

-- 初期データ: adminロールに全パーミッションを付与
INSERT INTO fw_m_role_permission (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM fw_m_role r, fw_m_permission p
WHERE r.role_name = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 初期データ: userロールに読み取りパーミッションを付与
INSERT INTO fw_m_role_permission (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM fw_m_role r, fw_m_permission p
WHERE r.role_name = 'user' AND p.operation = 'read'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 初期データ: viewerロールに読み取りパーミッションを付与
INSERT INTO fw_m_role_permission (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM fw_m_role r, fw_m_permission p
WHERE r.role_name = 'viewer' AND p.operation = 'read'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- リフレッシュトークンテーブル
CREATE TABLE IF NOT EXISTS fw_t_refresh_token (
    token_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES fw_m_user(user_id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_fw_t_refresh_token_user ON fw_t_refresh_token(user_id);
CREATE INDEX idx_fw_t_refresh_token_hash ON fw_t_refresh_token(token_hash);
CREATE INDEX idx_fw_t_refresh_token_expires ON fw_t_refresh_token(expires_at);

COMMENT ON TABLE fw_t_refresh_token IS 'リフレッシュトークン';
COMMENT ON COLUMN fw_t_refresh_token.token_id IS 'トークンID';
COMMENT ON COLUMN fw_t_refresh_token.user_id IS 'ユーザーID';
COMMENT ON COLUMN fw_t_refresh_token.token_hash IS 'トークンハッシュ';
COMMENT ON COLUMN fw_t_refresh_token.expires_at IS '有効期限';
COMMENT ON COLUMN fw_t_refresh_token.revoked_at IS '無効化日時';
