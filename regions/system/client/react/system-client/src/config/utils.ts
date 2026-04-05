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

/**
 * フィールド値をスキーマに基づいてバリデーションする。
 * string 型の場合は ReDoS 対策として、パターン長と入力長を制限した上で
 * try-catch で RegExp 生成エラーを捕捉する（M-15 監査対応）。
 */
export function validateFieldValue(schema: ConfigFieldSchema, value: unknown): string | undefined {
  switch (schema.type) {
    case 'integer':
      if (!Number.isInteger(value)) return '整数を入力してください';
      return validateNumberRange(schema, value as number);
    case 'float':
      if (typeof value !== 'number' || Number.isNaN(value)) return '数値を入力してください';
      return validateNumberRange(schema, value);
    case 'string': {
      if (typeof value !== 'string') return '文字列を入力してください';
      if (schema.pattern && value) {
        // ReDoS 対策: パターン長と入力長を制限（M-15 監査対応）
        // パターンが 500 文字超の場合は検証をスキップして安全側に倒す
        if (schema.pattern.length > 500) return undefined;
        // 入力値が 1024 文字超の場合は検証失敗として扱う
        if (value.length > 1024) return `パターン ${schema.pattern} に一致しません`;
        try {
          const regex = new RegExp(schema.pattern);
          if (!regex.test(value)) {
            return `パターン ${schema.pattern} に一致しません`;
          }
        } catch {
          // 不正なパターンでの RegExp 生成エラーは検証失敗として扱う
          return `パターン ${schema.pattern} に一致しません`;
        }
      }
      return undefined;
    }
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

/**
 * 2つの値が深く等価かどうかを判定する。
 * JSON.stringify による比較はキー順序やシンボル、undefined の扱いで
 * 誤った結果を返す場合があるため、再帰的な比較に変更（M-14 監査対応）。
 */
export function isEqualValue(left: unknown, right: unknown): boolean {
  return deepEqual(left, right);
}

/**
 * ライブラリに依存しない深い等価比較（JSON.stringify の限界を回避）。
 * プリミティブ値は参照等価（===）で比較し、オブジェクト・配列は
 * キーを再帰的に比較する。WeakSet により循環参照を検出して false を返す。
 */
function deepEqual(a: unknown, b: unknown, visited = new WeakSet()): boolean {
  if (a === b) return true;
  if (a === null || b === null) return false;
  if (typeof a !== 'object' || typeof b !== 'object') return false;
  // 循環参照を検出するためのWeakSet（設定値オブジェクトの安全な比較）
  if (visited.has(a as object)) return false;
  visited.add(a as object);
  // 配列とオブジェクトを混在させない
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  const keysA = Object.keys(a as object);
  const keysB = Object.keys(b as object);
  if (keysA.length !== keysB.length) return false;
  return keysA.every(key =>
    Object.prototype.hasOwnProperty.call(b, key) &&
    deepEqual((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key], visited)
  );
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
