-- config-service: 設定テーブル
-- 所有テーブル: fw_m_setting

-- 設定マスタ
CREATE TABLE IF NOT EXISTS fw_m_setting (
    setting_id BIGSERIAL PRIMARY KEY,
    setting_key VARCHAR(255) NOT NULL,
    setting_value TEXT NOT NULL,
    value_type VARCHAR(50) NOT NULL DEFAULT 'string',  -- string, number, boolean, json
    description TEXT,
    category VARCHAR(100),
    service_name VARCHAR(100),  -- NULLは全サービス共通
    environment VARCHAR(50),    -- NULLは全環境共通
    is_sensitive BOOLEAN NOT NULL DEFAULT FALSE,  -- 機密値（ログ出力時にマスク）
    is_readonly BOOLEAN NOT NULL DEFAULT FALSE,   -- 読み取り専用
    version INTEGER NOT NULL DEFAULT 1,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_to TIMESTAMPTZ,  -- NULLは無期限
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by BIGINT,
    updated_by BIGINT,
    CONSTRAINT uq_fw_m_setting_key UNIQUE (setting_key, service_name, environment)
);

-- 設定インデックス
CREATE INDEX idx_fw_m_setting_key ON fw_m_setting(setting_key);
CREATE INDEX idx_fw_m_setting_category ON fw_m_setting(category);
CREATE INDEX idx_fw_m_setting_service ON fw_m_setting(service_name);
CREATE INDEX idx_fw_m_setting_env ON fw_m_setting(environment);
CREATE INDEX idx_fw_m_setting_valid ON fw_m_setting(valid_from, valid_to);

-- 設定コメント
COMMENT ON TABLE fw_m_setting IS '設定マスタ';
COMMENT ON COLUMN fw_m_setting.setting_id IS '設定ID';
COMMENT ON COLUMN fw_m_setting.setting_key IS '設定キー';
COMMENT ON COLUMN fw_m_setting.setting_value IS '設定値';
COMMENT ON COLUMN fw_m_setting.value_type IS '値の型 (string, number, boolean, json)';
COMMENT ON COLUMN fw_m_setting.description IS '説明';
COMMENT ON COLUMN fw_m_setting.category IS 'カテゴリ';
COMMENT ON COLUMN fw_m_setting.service_name IS 'サービス名（NULLは全サービス共通）';
COMMENT ON COLUMN fw_m_setting.environment IS '環境（NULLは全環境共通）';
COMMENT ON COLUMN fw_m_setting.is_sensitive IS '機密フラグ';
COMMENT ON COLUMN fw_m_setting.is_readonly IS '読み取り専用フラグ';
COMMENT ON COLUMN fw_m_setting.version IS 'バージョン';
COMMENT ON COLUMN fw_m_setting.valid_from IS '有効開始日時';
COMMENT ON COLUMN fw_m_setting.valid_to IS '有効終了日時';

-- 設定履歴
CREATE TABLE IF NOT EXISTS fw_h_setting (
    history_id BIGSERIAL PRIMARY KEY,
    setting_id BIGINT NOT NULL,
    setting_key VARCHAR(255) NOT NULL,
    setting_value TEXT NOT NULL,
    value_type VARCHAR(50) NOT NULL,
    service_name VARCHAR(100),
    environment VARCHAR(50),
    version INTEGER NOT NULL,
    operation VARCHAR(20) NOT NULL,  -- INSERT, UPDATE, DELETE
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    changed_by BIGINT
);

-- 履歴インデックス
CREATE INDEX idx_fw_h_setting_id ON fw_h_setting(setting_id);
CREATE INDEX idx_fw_h_setting_key ON fw_h_setting(setting_key);
CREATE INDEX idx_fw_h_setting_changed ON fw_h_setting(changed_at);

-- 履歴コメント
COMMENT ON TABLE fw_h_setting IS '設定履歴';
COMMENT ON COLUMN fw_h_setting.history_id IS '履歴ID';
COMMENT ON COLUMN fw_h_setting.setting_id IS '設定ID';
COMMENT ON COLUMN fw_h_setting.operation IS '操作種別 (INSERT, UPDATE, DELETE)';
COMMENT ON COLUMN fw_h_setting.changed_at IS '変更日時';
COMMENT ON COLUMN fw_h_setting.changed_by IS '変更者';

-- 更新日時トリガー関数（auth-serviceで定義済みの場合はスキップ）
CREATE OR REPLACE FUNCTION fw_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 設定更新トリガー
CREATE TRIGGER tr_fw_m_setting_updated
    BEFORE UPDATE ON fw_m_setting
    FOR EACH ROW
    EXECUTE FUNCTION fw_update_timestamp();

-- 設定履歴記録関数
CREATE OR REPLACE FUNCTION fw_setting_history()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO fw_h_setting (
            setting_id, setting_key, setting_value, value_type,
            service_name, environment, version, operation, changed_by
        ) VALUES (
            OLD.setting_id, OLD.setting_key, OLD.setting_value, OLD.value_type,
            OLD.service_name, OLD.environment, OLD.version, 'DELETE', OLD.updated_by
        );
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO fw_h_setting (
            setting_id, setting_key, setting_value, value_type,
            service_name, environment, version, operation, changed_by
        ) VALUES (
            NEW.setting_id, NEW.setting_key, NEW.setting_value, NEW.value_type,
            NEW.service_name, NEW.environment, NEW.version, 'UPDATE', NEW.updated_by
        );
        RETURN NEW;
    ELSIF TG_OP = 'INSERT' THEN
        INSERT INTO fw_h_setting (
            setting_id, setting_key, setting_value, value_type,
            service_name, environment, version, operation, changed_by
        ) VALUES (
            NEW.setting_id, NEW.setting_key, NEW.setting_value, NEW.value_type,
            NEW.service_name, NEW.environment, NEW.version, 'INSERT', NEW.created_by
        );
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- 設定履歴トリガー
CREATE TRIGGER tr_fw_m_setting_history
    AFTER INSERT OR UPDATE OR DELETE ON fw_m_setting
    FOR EACH ROW
    EXECUTE FUNCTION fw_setting_history();

-- 初期データ: システム設定
INSERT INTO fw_m_setting (setting_key, setting_value, value_type, category, description, is_readonly) VALUES
    ('system.name', 'k1s0', 'string', 'system', 'システム名', TRUE),
    ('system.version', '0.1.0', 'string', 'system', 'システムバージョン', TRUE),
    ('system.maintenance_mode', 'false', 'boolean', 'system', 'メンテナンスモード', FALSE),
    ('auth.session_timeout_minutes', '60', 'number', 'auth', 'セッションタイムアウト（分）', FALSE),
    ('auth.max_failed_login_attempts', '5', 'number', 'auth', '最大ログイン失敗回数', FALSE),
    ('auth.password_min_length', '8', 'number', 'auth', 'パスワード最小文字数', FALSE),
    ('auth.token_expiry_seconds', '3600', 'number', 'auth', 'トークン有効期間（秒）', FALSE),
    ('logging.level', 'info', 'string', 'logging', 'ログレベル', FALSE),
    ('logging.format', 'json', 'string', 'logging', 'ログフォーマット', FALSE)
ON CONFLICT (setting_key, service_name, environment) DO NOTHING;
