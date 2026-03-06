-- Remove domain_scope from change_logs
DROP INDEX IF EXISTS master_maintenance.idx_change_logs_domain_scope;

ALTER TABLE master_maintenance.change_logs
    DROP COLUMN IF EXISTS domain_scope;

-- Restore original unique constraint on table_definitions
DROP INDEX IF EXISTS master_maintenance.uq_table_definitions_name_domain;
DROP INDEX IF EXISTS master_maintenance.idx_table_definitions_domain_scope;

ALTER TABLE master_maintenance.table_definitions
    ADD CONSTRAINT table_definitions_name_key UNIQUE (name);

ALTER TABLE master_maintenance.table_definitions
    DROP COLUMN IF EXISTS domain_scope;
