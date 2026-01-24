-- fw_m_endpoint: framework managed endpoints (PostgreSQL)

CREATE TABLE IF NOT EXISTS fw_m_endpoint (
	endpoint_id SERIAL PRIMARY KEY,
	service_name VARCHAR(100) NOT NULL,
	path VARCHAR(255) NOT NULL,
	method VARCHAR(10) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT fw_m_endpoint_service_path_method_uk UNIQUE (service_name, path, method)
);

CREATE INDEX IF NOT EXISTS fw_m_endpoint_service_name_idx ON fw_m_endpoint (service_name);
CREATE INDEX IF NOT EXISTS fw_m_endpoint_method_idx ON fw_m_endpoint (method);

