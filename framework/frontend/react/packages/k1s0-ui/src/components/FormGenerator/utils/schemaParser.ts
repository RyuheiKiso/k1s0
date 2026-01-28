/**
 * Zod スキーマ解析ユーティリティ
 */

import { z } from 'zod';
import type { ParsedFieldInfo, MuiFieldComponent } from '../types.js';

/**
 * Zod 型から MUI コンポーネントを推論する
 */
function inferComponent(
  zodType: string,
  info: Partial<ParsedFieldInfo>
): MuiFieldComponent {
  switch (zodType) {
    case 'ZodString':
      if (info.isEmail) return 'TextField';
      if (info.isUrl) return 'TextField';
      return 'TextField';

    case 'ZodNumber':
      return 'TextField';

    case 'ZodBoolean':
      return 'Switch';

    case 'ZodEnum':
      // 選択肢が少ない場合は RadioGroup
      if (info.enumValues && info.enumValues.length <= 4) {
        return 'RadioGroup';
      }
      return 'Select';

    case 'ZodDate':
      return 'DatePicker';

    case 'ZodArray':
      return 'TextField'; // ArrayField で処理

    case 'ZodObject':
      return 'TextField'; // ObjectField で処理

    default:
      return 'TextField';
  }
}

/**
 * ZodString のチェックを解析
 */
function parseStringChecks(schema: z.ZodString): Partial<ParsedFieldInfo> {
  const info: Partial<ParsedFieldInfo> = {};

  // 内部のチェックを取得
  const checks = (schema as { _def: { checks: Array<{ kind: string; value?: number; regex?: RegExp }> } })._def.checks || [];

  for (const check of checks) {
    switch (check.kind) {
      case 'min':
        info.min = check.value;
        break;
      case 'max':
        info.max = check.value;
        break;
      case 'email':
        info.isEmail = true;
        break;
      case 'url':
        info.isUrl = true;
        break;
      case 'regex':
        info.pattern = check.regex?.source;
        break;
    }
  }

  return info;
}

/**
 * ZodNumber のチェックを解析
 */
function parseNumberChecks(schema: z.ZodNumber): Partial<ParsedFieldInfo> {
  const info: Partial<ParsedFieldInfo> = {};

  const checks = (schema as { _def: { checks: Array<{ kind: string; value?: number }> } })._def.checks || [];

  for (const check of checks) {
    switch (check.kind) {
      case 'min':
        info.min = check.value;
        break;
      case 'max':
        info.max = check.value;
        break;
    }
  }

  return info;
}

/**
 * 内部のスキーマを取得（Optional, Default, Nullable などをアンラップ）
 */
function unwrapSchema(schema: z.ZodTypeAny): {
  innerSchema: z.ZodTypeAny;
  optional: boolean;
  defaultValue?: unknown;
} {
  let current = schema;
  let optional = false;
  let defaultValue: unknown;

  // ZodOptional
  if (current instanceof z.ZodOptional) {
    optional = true;
    current = current.unwrap();
  }

  // ZodDefault
  if (current instanceof z.ZodDefault) {
    defaultValue = (current as unknown as { _def: { defaultValue: () => unknown } })._def.defaultValue();
    current = current.removeDefault();
  }

  // ZodNullable
  if (current instanceof z.ZodNullable) {
    optional = true;
    current = current.unwrap();
  }

  // 再帰的にアンラップ
  if (
    current instanceof z.ZodOptional ||
    current instanceof z.ZodDefault ||
    current instanceof z.ZodNullable
  ) {
    const inner = unwrapSchema(current);
    return {
      innerSchema: inner.innerSchema,
      optional: optional || inner.optional,
      defaultValue: defaultValue ?? inner.defaultValue,
    };
  }

  return { innerSchema: current, optional, defaultValue };
}

/**
 * Zod スキーマの型名を取得
 */
function getZodTypeName(schema: z.ZodTypeAny): string {
  return schema.constructor.name;
}

/**
 * 単一フィールドのスキーマを解析
 */
export function parseFieldSchema(
  name: string,
  schema: z.ZodTypeAny
): ParsedFieldInfo {
  const { innerSchema, optional, defaultValue } = unwrapSchema(schema);
  const zodType = getZodTypeName(innerSchema);

  let info: Partial<ParsedFieldInfo> = {
    name,
    zodType,
    required: !optional,
    optional,
    defaultValue,
  };

  // 型別の詳細解析
  switch (zodType) {
    case 'ZodString':
      info = { ...info, ...parseStringChecks(innerSchema as z.ZodString) };
      break;

    case 'ZodNumber':
      info = { ...info, ...parseNumberChecks(innerSchema as z.ZodNumber) };
      break;

    case 'ZodEnum': {
      const enumSchema = innerSchema as z.ZodEnum<[string, ...string[]]>;
      info.enumValues = enumSchema.options as string[];
      break;
    }

    case 'ZodArray': {
      const arraySchema = innerSchema as z.ZodArray<z.ZodTypeAny>;
      info.arrayItemSchema = arraySchema.element;
      break;
    }

    case 'ZodObject': {
      info.objectSchema = innerSchema as z.ZodObject<z.ZodRawShape>;
      break;
    }
  }

  // コンポーネントを推論
  const inferredComponent = inferComponent(zodType, info);

  return {
    ...info,
    inferredComponent,
  } as ParsedFieldInfo;
}

/**
 * Zod オブジェクトスキーマを解析
 */
export function parseSchema<T extends z.ZodObject<z.ZodRawShape>>(
  schema: T
): Record<string, ParsedFieldInfo> {
  const shape = schema.shape;
  const result: Record<string, ParsedFieldInfo> = {};

  for (const [key, fieldSchema] of Object.entries(shape)) {
    result[key] = parseFieldSchema(key, fieldSchema as z.ZodTypeAny);
  }

  return result;
}

/**
 * スキーマからデフォルト値を抽出
 */
export function extractDefaultValues<T extends z.ZodObject<z.ZodRawShape>>(
  schema: T
): Partial<z.infer<T>> {
  const parsed = parseSchema(schema);
  const defaults: Record<string, unknown> = {};

  for (const [key, info] of Object.entries(parsed)) {
    if (info.defaultValue !== undefined) {
      defaults[key] = info.defaultValue;
    }
  }

  return defaults as Partial<z.infer<T>>;
}
