-- fw_m_role_permission: role-permission mapping (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_role_permission (
	role_id BIGINT NOT NULL,
	permission_id BIGINT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT fw_m_role_permission_pk PRIMARY KEY (role_id, permission_id),
	CONSTRAINT fw_m_role_permission_role_id_fk FOREIGN KEY (role_id) REFERENCES fw_m_role (role_id),
	CONSTRAINT fw_m_role_permission_permission_id_fk FOREIGN KEY (permission_id) REFERENCES fw_m_permission (permission_id)
);

CREATE INDEX IF NOT EXISTS fw_m_role_permission_permission_id_idx ON fw_m_role_permission (permission_id);
