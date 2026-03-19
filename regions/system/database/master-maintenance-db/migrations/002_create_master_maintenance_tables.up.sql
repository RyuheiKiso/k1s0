CREATE TABLE IF NOT EXISTS master_maintenance.table_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    schema_name VARCHAR(100) NOT NULL,
    database_name VARCHAR(100) NOT NULL DEFAULT 'default',
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    allow_create BOOLEAN NOT NULL DEFAULT TRUE,
    allow_update BOOLEAN NOT NULL DEFAULT TRUE,
    allow_delete BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_table_definitions_category
    ON master_maintenance.table_definitions(category);

CREATE INDEX IF NOT EXISTS idx_table_definitions_active
    ON master_maintenance.table_definitions(is_active);

CREATE TABLE IF NOT EXISTS master_maintenance.column_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    column_name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    data_type VARCHAR(50) NOT NULL,
    is_primary_key BOOLEAN NOT NULL DEFAULT FALSE,
    is_nullable BOOLEAN NOT NULL DEFAULT TRUE,
    is_unique BOOLEAN NOT NULL DEFAULT FALSE,
    default_value TEXT,
    max_length INTEGER,
    min_value DOUBLE PRECISION,
    max_value DOUBLE PRECISION,
    regex_pattern TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_searchable BOOLEAN NOT NULL DEFAULT FALSE,
    is_sortable BOOLEAN NOT NULL DEFAULT TRUE,
    is_filterable BOOLEAN NOT NULL DEFAULT FALSE,
    is_visible_in_list BOOLEAN NOT NULL DEFAULT TRUE,
    is_visible_in_form BOOLEAN NOT NULL DEFAULT TRUE,
    is_readonly BOOLEAN NOT NULL DEFAULT FALSE,
    input_type VARCHAR(50) NOT NULL DEFAULT 'text',
    select_options JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_column_definitions_table_column UNIQUE (table_id, column_name)
);

CREATE INDEX IF NOT EXISTS idx_column_definitions_table
    ON master_maintenance.column_definitions(table_id);

CREATE TABLE IF NOT EXISTS master_maintenance.table_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    source_column VARCHAR(255) NOT NULL,
    target_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    target_column VARCHAR(255) NOT NULL,
    relationship_type VARCHAR(50) NOT NULL,
    display_name VARCHAR(255),
    is_cascade_delete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_table_relationships_source
    ON master_maintenance.table_relationships(source_table_id);

CREATE INDEX IF NOT EXISTS idx_table_relationships_target
    ON master_maintenance.table_relationships(target_table_id);

CREATE TABLE IF NOT EXISTS master_maintenance.consistency_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    source_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    evaluation_timing VARCHAR(50) NOT NULL,
    error_message_template TEXT NOT NULL,
    zen_rule_json JSONB,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_consistency_rules_source_table
    ON master_maintenance.consistency_rules(source_table_id);

CREATE INDEX IF NOT EXISTS idx_consistency_rules_active
    ON master_maintenance.consistency_rules(is_active);

CREATE TABLE IF NOT EXISTS master_maintenance.rule_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES master_maintenance.consistency_rules(id) ON DELETE CASCADE,
    condition_order INTEGER NOT NULL,
    left_table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    left_column VARCHAR(255) NOT NULL,
    operator VARCHAR(50) NOT NULL,
    right_table_id UUID REFERENCES master_maintenance.table_definitions(id) ON DELETE SET NULL,
    right_column VARCHAR(255),
    right_value TEXT,
    logical_connector VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rule_conditions_rule
    ON master_maintenance.rule_conditions(rule_id, condition_order);

CREATE TABLE IF NOT EXISTS master_maintenance.display_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    config_type VARCHAR(50) NOT NULL,
    config_json JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_display_configs_table
    ON master_maintenance.display_configs(table_id);

CREATE TABLE IF NOT EXISTS master_maintenance.change_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_table VARCHAR(255) NOT NULL,
    target_record_id VARCHAR(255) NOT NULL,
    operation VARCHAR(50) NOT NULL,
    before_data JSONB,
    after_data JSONB,
    changed_columns TEXT[],
    changed_by VARCHAR(255) NOT NULL,
    change_reason TEXT,
    trace_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_change_logs_table
    ON master_maintenance.change_logs(target_table, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_logs_record
    ON master_maintenance.change_logs(target_table, target_record_id, created_at DESC);

CREATE TABLE IF NOT EXISTS master_maintenance.import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_id UUID NOT NULL REFERENCES master_maintenance.table_definitions(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    processed_rows INTEGER NOT NULL DEFAULT 0,
    error_rows INTEGER NOT NULL DEFAULT 0,
    error_details JSONB,
    started_by VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_import_jobs_table
    ON master_maintenance.import_jobs(table_id, started_at DESC);
