export type ConfigFieldType =
  | 'string' | 'integer' | 'float' | 'boolean'
  | 'enum' | 'object' | 'array';

export interface ConfigFieldSchema {
  key: string;
  label: string;
  description?: string;
  type: ConfigFieldType;
  min?: number;
  max?: number;
  options?: string[];
  pattern?: string;
  unit?: string;
  default?: unknown;
}

export interface ConfigCategorySchema {
  id: string;
  label: string;
  icon?: string;
  namespaces: string[];
  fields: ConfigFieldSchema[];
}

export interface ConfigEditorSchema {
  service: string;
  namespace_prefix: string;
  categories: ConfigCategorySchema[];
  updated_at?: string;
}

export interface ConfigFieldValue {
  id: string;
  key: string;
  namespace: string;
  value: unknown;
  originalValue: unknown;
  version: number;
  originalVersion: number;
  isDirty: boolean;
  hasError?: string;
}

export interface ConfigEditorConfig {
  service: string;
  categories: Array<ConfigCategorySchema & {
    fieldValues: Record<string, ConfigFieldValue>;
  }>;
  dirtyCount: number;
}

export interface ServiceConfigEntryResponse {
  namespace: string;
  key: string;
  value: unknown;
  version: number;
}

export interface ServiceConfigResultResponse {
  service_name: string;
  entries: ServiceConfigEntryResponse[];
}
