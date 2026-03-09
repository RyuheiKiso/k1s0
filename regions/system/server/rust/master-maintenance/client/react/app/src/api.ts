import axios from "axios";
import type {
  AuditLog,
  ColumnDefinition,
  ConsistencyRule,
  DisplayConfig,
  ImportJob,
  RuleResult,
  TableDefinition,
  TableRelationship,
  TableListResponse,
} from "./types";

const http = axios.create({
  baseURL: "/api/v1",
  headers: {
    "Content-Type": "application/json",
  },
});

export async function listTables(): Promise<TableListResponse> {
  const { data } = await http.get<TableListResponse>("/tables");
  return data;
}

export async function getTable(name: string): Promise<TableDefinition> {
  const { data } = await http.get<TableDefinition>(`/tables/${name}`);
  return data;
}

export async function getColumns(name: string): Promise<ColumnDefinition[]> {
  const { data } = await http.get<ColumnDefinition[]>(`/tables/${name}/columns`);
  return data;
}

export async function createTable(input: {
  name: string;
  schema_name: string;
  database_name?: string;
  display_name: string;
  description?: string;
  category?: string;
  allow_create?: boolean;
  allow_update?: boolean;
  allow_delete?: boolean;
  domain_scope?: string;
  read_roles?: string[];
  write_roles?: string[];
  admin_roles?: string[];
  sort_order?: number;
}): Promise<TableDefinition> {
  const { data } = await http.post<TableDefinition>("/tables", input);
  return data;
}

export async function createColumns(
  tableName: string,
  columns: Array<{
    column_name: string;
    display_name: string;
    data_type: string;
    input_type?: string;
    is_primary_key?: boolean;
    is_nullable?: boolean;
    is_unique?: boolean;
    is_visible_in_list?: boolean;
    is_visible_in_form?: boolean;
    is_searchable?: boolean;
    is_sortable?: boolean;
    is_filterable?: boolean;
    is_readonly?: boolean;
    default_value?: string;
    display_order?: number;
  }>
): Promise<ColumnDefinition[]> {
  const { data } = await http.post<ColumnDefinition[]>(`/tables/${tableName}/columns`, { columns });
  return data;
}

export async function getRecords(name: string): Promise<{ records: Record<string, unknown>[]; total: number }> {
  const { data } = await http.get(`/tables/${name}/records`);
  return data;
}

export async function createRecord(
  tableName: string,
  payload: Record<string, unknown>
): Promise<{ data: Record<string, unknown> }> {
  const { data } = await http.post<{ data: Record<string, unknown> }>(`/tables/${tableName}/records`, payload);
  return data;
}

export async function updateRecord(
  tableName: string,
  recordId: string,
  payload: Record<string, unknown>
): Promise<{ data: Record<string, unknown> }> {
  const { data } = await http.put<{ data: Record<string, unknown> }>(
    `/tables/${tableName}/records/${recordId}`,
    payload
  );
  return data;
}

export async function deleteRecord(tableName: string, recordId: string): Promise<void> {
  await http.delete(`/tables/${tableName}/records/${recordId}`);
}

export async function checkRules(name: string): Promise<{ results: RuleResult[] }> {
  const { data } = await http.post<{ results: RuleResult[] }>("/rules/check", { table_name: name });
  return data;
}

export async function listRules(tableName?: string): Promise<ConsistencyRule[]> {
  const { data } = await http.get<ConsistencyRule[]>("/rules", {
    params: tableName ? { table: tableName } : undefined,
  });
  return data;
}

export async function createRule(input: {
  name: string;
  description?: string;
  rule_type: string;
  severity?: string;
  source_table: string;
  evaluation_timing?: string;
  error_message_template: string;
  conditions?: Array<{
    condition_order: number;
    left_column: string;
    operator: string;
    right_table?: string;
    right_column?: string;
    right_value?: string;
    logical_connector?: string;
  }>;
}): Promise<ConsistencyRule> {
  const { data } = await http.post<ConsistencyRule>("/rules", input);
  return data;
}

export async function listRelationships(): Promise<TableRelationship[]> {
  const { data } = await http.get<TableRelationship[]>("/relationships");
  return data;
}

export async function createRelationship(input: {
  source_table: string;
  source_column: string;
  target_table: string;
  target_column: string;
  relationship_type: string;
  display_name?: string;
  is_cascade_delete?: boolean;
}): Promise<TableRelationship> {
  const { data } = await http.post<TableRelationship>("/relationships", input);
  return data;
}

export async function updateRelationship(
  id: string,
  input: {
    source_column?: string;
    target_column?: string;
    relationship_type?: string;
    display_name?: string;
    is_cascade_delete?: boolean;
  }
): Promise<TableRelationship> {
  const { data } = await http.put<TableRelationship>(`/relationships/${id}`, input);
  return data;
}

export async function deleteRelationship(id: string): Promise<void> {
  await http.delete(`/relationships/${id}`);
}

export async function getTableAuditLogs(tableName: string): Promise<{ logs: AuditLog[]; total: number }> {
  const { data } = await http.get<{ logs: AuditLog[]; total: number }>(`/tables/${tableName}/audit-logs`);
  return data;
}

export async function listDisplayConfigs(tableName: string): Promise<DisplayConfig[]> {
  const { data } = await http.get<DisplayConfig[]>(`/tables/${tableName}/display-configs`);
  return data;
}

export async function createDisplayConfig(
  tableName: string,
  input: { config_type: string; config_json: unknown; is_default?: boolean }
): Promise<DisplayConfig> {
  const { data } = await http.post<DisplayConfig>(`/tables/${tableName}/display-configs`, input);
  return data;
}

export async function updateDisplayConfig(
  tableName: string,
  id: string,
  input: { config_type?: string; config_json?: unknown; is_default?: boolean }
): Promise<DisplayConfig> {
  const { data } = await http.put<DisplayConfig>(`/tables/${tableName}/display-configs/${id}`, input);
  return data;
}

export async function deleteDisplayConfig(tableName: string, id: string): Promise<void> {
  await http.delete(`/tables/${tableName}/display-configs/${id}`);
}

export async function getImportJob(id: string): Promise<ImportJob> {
  const { data } = await http.get<ImportJob>(`/import-jobs/${id}`);
  return data;
}

export async function uploadImportFile(tableName: string, file: File): Promise<ImportJob> {
  const formData = new FormData();
  formData.append("file", file);
  const { data } = await http.post<ImportJob>(`/tables/${tableName}/import-file`, formData, {
    headers: {
      "Content-Type": "multipart/form-data",
    },
  });
  return data;
}

export async function exportCsv(tableName: string): Promise<string> {
  const { data } = await http.get<string>(`/tables/${tableName}/export`, {
    params: { format: "csv" },
    responseType: "text",
  });
  return data;
}
