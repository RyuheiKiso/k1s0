-- fw_m_setting: framework managed settings (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_setting (
	setting_id BIGSERIAL PRIMARY KEY,
	service_name VARCHAR(100) NOT NULL,
	env VARCHAR(20) NOT NULL DEFAULT 'default',
	setting_key VARCHAR(150) NOT NULL,
	value_type VARCHAR(30) NOT NULL DEFAULT 'string',
	setting_value TEXT NULL,
	description TEXT NULL,
	status SMALLINT NOT NULL DEFAULT 1,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	deleted_at TIMESTAMPTZ NULL,
	CONSTRAINT fw_m_setting_service_name_env_setting_key_uk UNIQUE (service_name, env, setting_key)
);

CREATE INDEX IF NOT EXISTS fw_m_setting_service_name_idx ON fw_m_setting (service_name);
CREATE INDEX IF NOT EXISTS fw_m_setting_env_idx ON fw_m_setting (env);
CREATE INDEX IF NOT EXISTS fw_m_setting_status_idx ON fw_m_setting (status);
CREATE INDEX IF NOT EXISTS fw_m_setting_deleted_at_idx ON fw_m_setting (deleted_at);

