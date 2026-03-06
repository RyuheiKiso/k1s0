-- Add domain_scope to table_definitions
ALTER TABLE master_maintenance.table_definitions
    ADD COLUMN domain_scope VARCHAR(100) DEFAULT NULL;

-- Replace unique constraint on name with composite unique index
ALTER TABLE master_maintenance.table_definitions
    DROP CONSTRAINT table_definitions_name_key;

CREATE UNIQUE INDEX uq_table_definitions_name_domain
    ON master_maintenance.table_definitions (name, COALESCE(domain_scope, '__system__'));

-- Add index for domain_scope filtering
CREATE INDEX idx_table_definitions_domain_scope
    ON master_maintenance.table_definitions(domain_scope);

-- Add domain_scope to change_logs
ALTER TABLE master_maintenance.change_logs
    ADD COLUMN domain_scope VARCHAR(100) DEFAULT NULL;

CREATE INDEX idx_change_logs_domain_scope
    ON master_maintenance.change_logs(domain_scope);
