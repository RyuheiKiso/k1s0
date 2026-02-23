import type { AxiosInstance } from 'axios';
import type { ConfigEditorSchema, ConfigEditorConfig, ConfigFieldValue } from './types';

interface ConfigValueResponse {
  namespace: string;
  key: string;
  value: unknown;
}

export class ConfigInterpreter {
  constructor(private readonly client: AxiosInstance) {}

  async build(serviceName: string): Promise<ConfigEditorConfig> {
    const [schemaRes, valuesRes] = await Promise.all([
      this.client.get<ConfigEditorSchema>(`/api/v1/config-schema/${serviceName}`),
      this.client.get<ConfigValueResponse[]>(`/api/v1/config/services/${serviceName}`),
    ]);

    const schema = schemaRes.data;
    const values = valuesRes.data;

    const valueMap = new Map<string, ConfigValueResponse>();
    for (const v of values) {
      valueMap.set(v.key, v);
    }

    const categories = schema.categories.map((cat) => {
      const fieldValues: Record<string, ConfigFieldValue> = {};

      for (const field of cat.fields) {
        const existing = valueMap.get(field.key);
        const currentValue = existing?.value ?? field.default;
        fieldValues[field.key] = {
          key: field.key,
          namespace: existing?.namespace ?? cat.namespaces[0] ?? '',
          value: currentValue,
          originalValue: currentValue,
          isDirty: false,
        };
      }

      return { ...cat, fieldValues };
    });

    return {
      service: schema.service,
      categories,
      dirtyCount: 0,
    };
  }
}
