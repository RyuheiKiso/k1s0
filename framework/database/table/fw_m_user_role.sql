-- fw_m_user_role: user-role mapping (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_user_role (
	user_id BIGINT NOT NULL,
	role_id BIGINT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT fw_m_user_role_pk PRIMARY KEY (user_id, role_id),
	CONSTRAINT fw_m_user_role_user_id_fk FOREIGN KEY (user_id) REFERENCES fw_m_user (user_id),
	CONSTRAINT fw_m_user_role_role_id_fk FOREIGN KEY (role_id) REFERENCES fw_m_role (role_id)
);

CREATE INDEX IF NOT EXISTS fw_m_user_role_role_id_idx ON fw_m_user_role (role_id);
