import type { AxiosInstance } from 'axios';
import type {
  ConfigEditorConfig,
  ConfigEditorSchema,
  ConfigFieldValue,
  ServiceConfigEntryResponse,
  ServiceConfigResultResponse,
} from './types';
import { buildFieldId, findEntryForField, validateFieldValue } from './utils';

export class ConfigInterpreter {
  constructor(private readonly client: AxiosInstance) {}

  async build(serviceName: string): Promise<ConfigEditorConfig> {
    const [schemaRes, valuesRes] = await Promise.all([
      this.client.get<ConfigEditorSchema>(`/api/v1/config-schema/${serviceName}`),
      this.client.get<ServiceConfigResultResponse>(`/api/v1/config/services/${serviceName}`),
    ]);

    const schema = schemaRes.data;
    const valueMap = new Map<string, ServiceConfigEntryResponse>();
    for (const entry of valuesRes.data.entries) {
      valueMap.set(buildFieldId(entry.namespace, entry.key), entry);
    }

    const categories = schema.categories.map((category) => {
      const fieldValues: Record<string, ConfigFieldValue> = {};

      for (const field of category.fields) {
        const existing = findEntryForField(category.namespaces, field.key, valueMap);
        const namespace = existing?.namespace ?? category.namespaces[0] ?? schema.namespace_prefix;
        const currentValue = existing?.value ?? field.default;

        fieldValues[field.key] = {
          id: buildFieldId(namespace, field.key),
          key: field.key,
          namespace,
          value: currentValue,
          originalValue: currentValue,
          version: existing?.version ?? 0,
          originalVersion: existing?.version ?? 0,
          isDirty: false,
          hasError: validateFieldValue(field, currentValue),
        };
      }

      return { ...category, fieldValues };
    });

    return {
      service: valuesRes.data.service_name || schema.service,
      categories,
      dirtyCount: 0,
    };
  }
}
