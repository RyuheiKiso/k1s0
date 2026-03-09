ALTER TABLE master_maintenance.table_definitions
    ADD COLUMN read_roles TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN write_roles TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN admin_roles TEXT[] NOT NULL DEFAULT '{}';
