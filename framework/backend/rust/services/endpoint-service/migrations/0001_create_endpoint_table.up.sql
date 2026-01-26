-- endpoint-service: エンドポイントテーブル
-- 所有テーブル: fw_m_endpoint

-- エンドポイントマスタ
CREATE TABLE IF NOT EXISTS fw_m_endpoint (
    endpoint_id BIGSERIAL PRIMARY KEY,
    service_name VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    method VARCHAR(20) NOT NULL,  -- GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD
    protocol VARCHAR(20) NOT NULL DEFAULT 'http',  -- http, https, grpc, grpcs
    description TEXT,
    version VARCHAR(20) DEFAULT 'v1',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,  -- 認証不要かどうか
    is_deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    deprecated_at TIMESTAMPTZ,
    deprecated_message TEXT,
    rate_limit_per_minute INTEGER,  -- レート制限（NULLは無制限）
    timeout_ms INTEGER DEFAULT 30000,  -- タイムアウト（ミリ秒）
    retry_count INTEGER DEFAULT 0,     -- リトライ回数
    metadata JSONB,  -- 追加メタデータ
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT,
    CONSTRAINT uq_fw_m_endpoint UNIQUE (service_name, path, method, version)
);

-- エンドポイントインデックス
CREATE INDEX idx_fw_m_endpoint_service ON fw_m_endpoint(service_name);
CREATE INDEX idx_fw_m_endpoint_path ON fw_m_endpoint(path);
CREATE INDEX idx_fw_m_endpoint_method ON fw_m_endpoint(method);
CREATE INDEX idx_fw_m_endpoint_protocol ON fw_m_endpoint(protocol);
CREATE INDEX idx_fw_m_endpoint_public ON fw_m_endpoint(is_public);
CREATE INDEX idx_fw_m_endpoint_deprecated ON fw_m_endpoint(is_deprecated);

-- エンドポイントコメント
COMMENT ON TABLE fw_m_endpoint IS 'エンドポイントマスタ';
COMMENT ON COLUMN fw_m_endpoint.endpoint_id IS 'エンドポイントID';
COMMENT ON COLUMN fw_m_endpoint.service_name IS 'サービス名';
COMMENT ON COLUMN fw_m_endpoint.path IS 'パス';
COMMENT ON COLUMN fw_m_endpoint.method IS 'HTTPメソッド';
COMMENT ON COLUMN fw_m_endpoint.protocol IS 'プロトコル';
COMMENT ON COLUMN fw_m_endpoint.description IS '説明';
COMMENT ON COLUMN fw_m_endpoint.version IS 'APIバージョン';
COMMENT ON COLUMN fw_m_endpoint.is_public IS '公開フラグ（認証不要）';
COMMENT ON COLUMN fw_m_endpoint.is_deprecated IS '非推奨フラグ';
COMMENT ON COLUMN fw_m_endpoint.rate_limit_per_minute IS 'レート制限（/分）';
COMMENT ON COLUMN fw_m_endpoint.timeout_ms IS 'タイムアウト（ミリ秒）';
COMMENT ON COLUMN fw_m_endpoint.retry_count IS 'リトライ回数';
COMMENT ON COLUMN fw_m_endpoint.metadata IS '追加メタデータ（JSON）';

-- サービスアドレスオーバーライド
CREATE TABLE IF NOT EXISTS fw_m_service_address (
    address_id BIGSERIAL PRIMARY KEY,
    service_name VARCHAR(255) NOT NULL,
    protocol VARCHAR(20) NOT NULL,
    address VARCHAR(512) NOT NULL,
    use_tls BOOLEAN NOT NULL DEFAULT FALSE,
    environment VARCHAR(50),  -- NULLは全環境共通
    priority INTEGER NOT NULL DEFAULT 0,  -- 高い値が優先
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    health_check_path VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT,
    CONSTRAINT uq_fw_m_service_address UNIQUE (service_name, protocol, environment, priority)
);

-- サービスアドレスインデックス
CREATE INDEX idx_fw_m_service_address_service ON fw_m_service_address(service_name);
CREATE INDEX idx_fw_m_service_address_protocol ON fw_m_service_address(protocol);
CREATE INDEX idx_fw_m_service_address_env ON fw_m_service_address(environment);
CREATE INDEX idx_fw_m_service_address_active ON fw_m_service_address(is_active);

-- サービスアドレスコメント
COMMENT ON TABLE fw_m_service_address IS 'サービスアドレスオーバーライド';
COMMENT ON COLUMN fw_m_service_address.address_id IS 'アドレスID';
COMMENT ON COLUMN fw_m_service_address.service_name IS 'サービス名';
COMMENT ON COLUMN fw_m_service_address.protocol IS 'プロトコル';
COMMENT ON COLUMN fw_m_service_address.address IS 'アドレス';
COMMENT ON COLUMN fw_m_service_address.use_tls IS 'TLS使用フラグ';
COMMENT ON COLUMN fw_m_service_address.environment IS '環境';
COMMENT ON COLUMN fw_m_service_address.priority IS '優先度';
COMMENT ON COLUMN fw_m_service_address.is_active IS '有効フラグ';
COMMENT ON COLUMN fw_m_service_address.health_check_path IS 'ヘルスチェックパス';

-- エンドポイント権限マッピング
CREATE TABLE IF NOT EXISTS fw_m_endpoint_permission (
    endpoint_id BIGINT NOT NULL REFERENCES fw_m_endpoint(endpoint_id) ON DELETE CASCADE,
    permission_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    PRIMARY KEY (endpoint_id, permission_key)
);

-- エンドポイント権限インデックス
CREATE INDEX idx_fw_m_endpoint_permission_endpoint ON fw_m_endpoint_permission(endpoint_id);
CREATE INDEX idx_fw_m_endpoint_permission_key ON fw_m_endpoint_permission(permission_key);

-- エンドポイント権限コメント
COMMENT ON TABLE fw_m_endpoint_permission IS 'エンドポイント権限マッピング';
COMMENT ON COLUMN fw_m_endpoint_permission.endpoint_id IS 'エンドポイントID';
COMMENT ON COLUMN fw_m_endpoint_permission.permission_key IS '必要なパーミッションキー';

-- 更新日時トリガー関数（他のサービスで定義済みの場合はスキップ）
CREATE OR REPLACE FUNCTION fw_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 更新日時トリガー
CREATE TRIGGER tr_fw_m_endpoint_updated
    BEFORE UPDATE ON fw_m_endpoint
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

CREATE TRIGGER tr_fw_m_service_address_updated
    BEFORE UPDATE ON fw_m_service_address
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

-- 初期データ: 共通サービスのエンドポイント
INSERT INTO fw_m_endpoint (service_name, path, method, protocol, description, is_public) VALUES
    -- Health checks (public)
    ('auth-service', '/healthz', 'GET', 'http', 'ヘルスチェック', TRUE),
    ('auth-service', '/readyz', 'GET', 'http', 'レディネスチェック', TRUE),
    ('config-service', '/healthz', 'GET', 'http', 'ヘルスチェック', TRUE),
    ('config-service', '/readyz', 'GET', 'http', 'レディネスチェック', TRUE),
    ('endpoint-service', '/healthz', 'GET', 'http', 'ヘルスチェック', TRUE),
    ('endpoint-service', '/readyz', 'GET', 'http', 'レディネスチェック', TRUE),
    -- Auth service endpoints
    ('auth-service', '/k1s0.auth.v1.AuthService/Authenticate', 'POST', 'grpc', '認証', TRUE),
    ('auth-service', '/k1s0.auth.v1.AuthService/RefreshToken', 'POST', 'grpc', 'トークンリフレッシュ', FALSE),
    ('auth-service', '/k1s0.auth.v1.AuthService/CheckPermission', 'POST', 'grpc', '権限チェック', FALSE),
    ('auth-service', '/k1s0.auth.v1.AuthService/GetUser', 'POST', 'grpc', 'ユーザー取得', FALSE),
    ('auth-service', '/k1s0.auth.v1.AuthService/ListUserRoles', 'POST', 'grpc', 'ユーザーロール一覧', FALSE),
    -- Config service endpoints
    ('config-service', '/k1s0.config.v1.ConfigService/GetSetting', 'POST', 'grpc', '設定取得', FALSE),
    ('config-service', '/k1s0.config.v1.ConfigService/ListSettings', 'POST', 'grpc', '設定一覧', FALSE),
    -- Endpoint service endpoints
    ('endpoint-service', '/k1s0.endpoint.v1.EndpointService/GetEndpoint', 'POST', 'grpc', 'エンドポイント取得', FALSE),
    ('endpoint-service', '/k1s0.endpoint.v1.EndpointService/ListEndpoints', 'POST', 'grpc', 'エンドポイント一覧', FALSE),
    ('endpoint-service', '/k1s0.endpoint.v1.EndpointService/ResolveEndpoint', 'POST', 'grpc', 'エンドポイント解決', FALSE)
ON CONFLICT (service_name, path, method, version) DO NOTHING;

-- 初期データ: エンドポイント権限マッピング
INSERT INTO fw_m_endpoint_permission (endpoint_id, permission_key)
SELECT e.endpoint_id, 'user:read'
FROM fw_m_endpoint e
WHERE e.service_name = 'auth-service'
  AND e.path = '/k1s0.auth.v1.AuthService/GetUser'
ON CONFLICT (endpoint_id, permission_key) DO NOTHING;
