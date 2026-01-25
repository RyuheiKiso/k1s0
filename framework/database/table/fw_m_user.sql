-- fw_m_user: framework managed users (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_user (
	user_id BIGSERIAL PRIMARY KEY,
	login_id VARCHAR(100) NOT NULL,
	email VARCHAR(255) NOT NULL,
	password_hash TEXT NULL,
	display_name VARCHAR(100) NOT NULL,
	status SMALLINT NOT NULL DEFAULT 1,
	last_login_at TIMESTAMPTZ NULL,
	password_updated_at TIMESTAMPTZ NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	deleted_at TIMESTAMPTZ NULL,
	CONSTRAINT fw_m_user_login_id_uk UNIQUE (login_id),
	CONSTRAINT fw_m_user_email_uk UNIQUE (email)
);

CREATE INDEX IF NOT EXISTS fw_m_user_status_idx ON fw_m_user (status);
CREATE INDEX IF NOT EXISTS fw_m_user_deleted_at_idx ON fw_m_user (deleted_at);

