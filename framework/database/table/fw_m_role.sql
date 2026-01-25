-- fw_m_role: framework managed roles (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_role (
	role_id BIGSERIAL PRIMARY KEY,
	role_name VARCHAR(100) NOT NULL,
	description TEXT NULL,
	status SMALLINT NOT NULL DEFAULT 1,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	deleted_at TIMESTAMPTZ NULL,
	CONSTRAINT fw_m_role_role_name_uk UNIQUE (role_name)
);

CREATE INDEX IF NOT EXISTS fw_m_role_status_idx ON fw_m_role (status);
CREATE INDEX IF NOT EXISTS fw_m_role_deleted_at_idx ON fw_m_role (deleted_at);

