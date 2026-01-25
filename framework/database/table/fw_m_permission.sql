-- fw_m_permission: framework managed permissions (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_permission (
	permission_id BIGSERIAL PRIMARY KEY,
	service_name VARCHAR(100) NOT NULL,
	permission_key VARCHAR(150) NOT NULL,
	description TEXT NULL,
	status SMALLINT NOT NULL DEFAULT 1,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	deleted_at TIMESTAMPTZ NULL,
	CONSTRAINT fw_m_permission_service_name_permission_key_uk UNIQUE (service_name, permission_key)
);

CREATE INDEX IF NOT EXISTS fw_m_permission_service_name_idx ON fw_m_permission (service_name);
CREATE INDEX IF NOT EXISTS fw_m_permission_status_idx ON fw_m_permission (status);
CREATE INDEX IF NOT EXISTS fw_m_permission_deleted_at_idx ON fw_m_permission (deleted_at);

