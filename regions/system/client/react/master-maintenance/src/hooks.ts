import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  checkRules,
  createRecord,
  createColumns,
  createDisplayConfig,
  createRelationship,
  createRule,
  createTable,
  deleteDisplayConfig,
  deleteRecord,
  deleteRelationship,
  exportCsv,
  getColumns,
  getRecords,
  getTableAuditLogs,
  listRules,
  listRelationships,
  listDisplayConfigs,
  listTables,
  updateDisplayConfig,
  updateRelationship,
  updateRecord,
  uploadImportFile,
} from "./api";

export function useTables() {
  return useQuery({
    queryKey: ["tables"],
    queryFn: listTables,
  });
}

export function useCreateTable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createTable,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["tables"] });
    },
  });
}

export function useColumns(tableName: string | undefined) {
  return useQuery({
    queryKey: ["columns", tableName],
    queryFn: () => getColumns(tableName!),
    enabled: Boolean(tableName),
  });
}

export function useCreateColumns(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (
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
    ) => createColumns(tableName!, columns),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["columns", tableName] });
      await queryClient.invalidateQueries({ queryKey: ["records", tableName] });
    },
  });
}

export function useRecords(tableName: string | undefined) {
  return useQuery({
    queryKey: ["records", tableName],
    queryFn: () => getRecords(tableName!),
    enabled: Boolean(tableName),
  });
}

export function useCreateRecord(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (payload: Record<string, unknown>) => createRecord(tableName!, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", tableName] });
    },
  });
}

export function useUpdateRecord(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ recordId, payload }: { recordId: string; payload: Record<string, unknown> }) =>
      updateRecord(tableName!, recordId, payload),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", tableName] });
    },
  });
}

export function useDeleteRecord(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (recordId: string) => deleteRecord(tableName!, recordId),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["records", tableName] });
    },
  });
}

export function useRuleCheck(tableName: string | undefined) {
  return useQuery({
    queryKey: ["rules", "check", tableName],
    queryFn: () => checkRules(tableName!),
    enabled: Boolean(tableName),
  });
}

export function useRelationships() {
  return useQuery({
    queryKey: ["relationships"],
    queryFn: listRelationships,
  });
}

export function useCreateRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createRelationship,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["relationships"] });
    },
  });
}

export function useDeleteRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteRelationship,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["relationships"] });
    },
  });
}

export function useUpdateRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      input,
    }: {
      id: string;
      input: {
        source_column?: string;
        target_column?: string;
        relationship_type?: string;
        display_name?: string;
        is_cascade_delete?: boolean;
      };
    }) => updateRelationship(id, input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["relationships"] });
    },
  });
}

export function useTableAuditLogs(tableName: string | undefined) {
  return useQuery({
    queryKey: ["audit-logs", tableName],
    queryFn: () => getTableAuditLogs(tableName!),
    enabled: Boolean(tableName),
  });
}

export function useDisplayConfigs(tableName: string | undefined) {
  return useQuery({
    queryKey: ["display-configs", tableName],
    queryFn: () => listDisplayConfigs(tableName!),
    enabled: Boolean(tableName),
  });
}

export function useCreateDisplayConfig(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { config_type: string; config_json: unknown; is_default?: boolean }) =>
      createDisplayConfig(tableName!, input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["display-configs", tableName] });
    },
  });
}

export function useUpdateDisplayConfig(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      input,
    }: {
      id: string;
      input: { config_type?: string; config_json?: unknown; is_default?: boolean };
    }) => updateDisplayConfig(tableName!, id, input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["display-configs", tableName] });
    },
  });
}

export function useDeleteDisplayConfig(tableName: string | undefined) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteDisplayConfig(tableName!, id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["display-configs", tableName] });
    },
  });
}

export function useRules(tableName: string | undefined) {
  return useQuery({
    queryKey: ["rules", tableName],
    queryFn: () => listRules(tableName),
  });
}

export function useCreateRule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createRule,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["rules"] });
    },
  });
}

export function useImportUpload(tableName: string | undefined) {
  return useMutation({
    mutationFn: (file: File) => uploadImportFile(tableName!, file),
  });
}

export function useCsvExport(tableName: string | undefined) {
  return useMutation({
    mutationFn: () => exportCsv(tableName!),
  });
}
