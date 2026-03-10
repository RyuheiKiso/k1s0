import type {
  ConfigEditorConfig,
  ConfigFieldSchema,
  ConfigFieldValue,
  ServiceConfigEntryResponse,
} from './types';

export function buildFieldId(namespace: string, key: string): string {
  return `${namespace}::${key}`;
}

export function cloneConfig(config: ConfigEditorConfig): ConfigEditorConfig {
  return {
    ...config,
    categories: config.categories.map((category) => ({
      ...category,
      fields: category.fields.map((field) => ({ ...field })),
      fieldValues: Object.fromEntries(
        Object.entries(category.fieldValues).map(([key, value]) => [key, { ...value }]),
      ),
    })),
  };
}

export function validateFieldValue(schema: ConfigFieldSchema, value: unknown): string | undefined {
  switch (schema.type) {
    case 'integer':
      if (!Number.isInteger(value)) return '整数を入力してください';
      return validateNumberRange(schema, value as number);
    case 'float':
      if (typeof value !== 'number' || Number.isNaN(value)) return '数値を入力してください';
      return validateNumberRange(schema, value);
    case 'string':
      if (typeof value !== 'string') return '文字列を入力してください';
      if (schema.pattern && value && !(new RegExp(schema.pattern).test(value))) {
        return `パターン ${schema.pattern} に一致しません`;
      }
      return undefined;
    case 'boolean':
      return typeof value === 'boolean' ? undefined : '真偽値を入力してください';
    case 'enum':
      if (typeof value !== 'string') return '文字列を選択してください';
      if (schema.options && !schema.options.includes(value)) {
        return '定義済みの候補を選択してください';
      }
      return undefined;
    case 'object':
      return isPlainObject(value) ? undefined : 'オブジェクトを入力してください';
    case 'array':
      return Array.isArray(value) ? undefined : '配列を入力してください';
    default:
      return undefined;
  }
}

export function findEntryForField(
  namespaces: string[],
  key: string,
  entries: Map<string, ServiceConfigEntryResponse>,
): ServiceConfigEntryResponse | undefined {
  for (const namespace of namespaces) {
    const entry = entries.get(buildFieldId(namespace, key));
    if (entry) {
      return entry;
    }
  }
  return undefined;
}

export function updateDirtyField(
  field: ConfigFieldValue,
  nextValue: unknown,
  nextError?: string,
): ConfigFieldValue {
  const isDirty = !isEqualValue(nextValue, field.originalValue);
  return {
    ...field,
    value: nextValue,
    isDirty,
    hasError: nextError,
  };
}

export function isEqualValue(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function validateNumberRange(schema: ConfigFieldSchema, value: number): string | undefined {
  if (schema.min !== undefined && value < schema.min) {
    return `${schema.min} 以上の値を入力してください`;
  }
  if (schema.max !== undefined && value > schema.max) {
    return `${schema.max} 以下の値を入力してください`;
  }
  return undefined;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
