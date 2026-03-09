export type TableDefinition = {
  id: string;
  name: string;
  schema_name: string;
  database_name: string;
  display_name: string;
  description?: string | null;
  category?: string | null;
  is_active: boolean;
  allow_create: boolean;
  allow_update: boolean;
  allow_delete: boolean;
  sort_order: number;
  created_by: string;
  created_at: string;
  updated_at: string;
  domain_scope?: string | null;
  read_roles?: string[];
  write_roles?: string[];
  admin_roles?: string[];
  columns?: ColumnDefinition[];
};

export type ColumnDefinition = {
  id: string;
  table_id: string;
  column_name: string;
  display_name: string;
  data_type: string;
  is_primary_key: boolean;
  is_nullable: boolean;
  is_unique: boolean;
  is_searchable: boolean;
  is_sortable: boolean;
  is_filterable: boolean;
  is_visible_in_list: boolean;
  is_visible_in_form: boolean;
  is_readonly: boolean;
  input_type: string;
  default_value?: string | null;
  max_length?: number | null;
  min_value?: number | null;
  max_value?: number | null;
  regex_pattern?: string | null;
  select_options?: unknown;
};

export type TableListResponse = {
  tables: TableDefinition[];
  pagination: {
    total_count: number;
    page: number;
    page_size: number;
    has_next: boolean;
  };
};

export type RuleResult = {
  rule_id: string;
  rule_name: string;
  passed: boolean;
  message?: string | null;
  severity: string;
  affected_record_ids: string[];
};

export type ImportJob = {
  id: string;
  table_id: string;
  file_name: string;
  status: string;
  total_rows: number;
  processed_rows: number;
  error_rows: number;
  started_by: string;
  started_at: string;
  completed_at?: string | null;
};

export type ConsistencyRule = {
  id: string;
  name: string;
  description?: string | null;
  rule_type: string;
  severity: string;
  is_active: boolean;
  source_table_id: string;
  evaluation_timing: string;
  error_message_template: string;
  zen_rule_json?: unknown;
  created_by: string;
  created_at: string;
  updated_at: string;
};

export type TableRelationship = {
  id: string;
  source_column: string;
  target_table: string;
  target_column: string;
  relationship_type: string;
  display_name?: string | null;
  is_cascade_delete: boolean;
  created_at: string;
};

export type AuditLog = {
  id: string;
  target_table: string;
  target_record_id: string;
  operation: string;
  before_data?: unknown;
  after_data?: unknown;
  changed_columns?: string[] | null;
  changed_by: string;
  change_reason?: string | null;
  trace_id?: string | null;
  created_at: string;
};

export type DisplayConfig = {
  id: string;
  table_id: string;
  config_type: string;
  config_json: unknown;
  is_default: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
};
