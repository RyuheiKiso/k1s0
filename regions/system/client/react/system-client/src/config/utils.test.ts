import { describe, it, expect } from 'vitest';
import {
  buildFieldId,
  validateFieldValue,
  findEntryForField,
  updateDirtyField,
  isEqualValue,
} from './utils';
import type {
  ConfigFieldSchema,
  ConfigFieldValue,
  ServiceConfigEntryResponse,
} from './types';

// buildFieldId テスト

describe('buildFieldId', () => {
  it('namespace と key を :: で連結する', () => {
    expect(buildFieldId('app.general', 'timeout')).toBe('app.general::timeout');
  });

  it('空文字列でも動作する', () => {
    expect(buildFieldId('', '')).toBe('::');
  });
});

// isEqualValue テスト

describe('isEqualValue', () => {
  it('同じプリミティブ値は等しい', () => {
    expect(isEqualValue(42, 42)).toBe(true);
    expect(isEqualValue('hello', 'hello')).toBe(true);
    expect(isEqualValue(true, true)).toBe(true);
  });

  it('異なるプリミティブ値は等しくない', () => {
    expect(isEqualValue(1, 2)).toBe(false);
    expect(isEqualValue('a', 'b')).toBe(false);
  });

  it('同じオブジェクトは等しい', () => {
    expect(isEqualValue({ a: 1 }, { a: 1 })).toBe(true);
  });

  it('異なるオブジェクトは等しくない', () => {
    expect(isEqualValue({ a: 1 }, { a: 2 })).toBe(false);
  });

  it('null と undefined は等しくない', () => {
    expect(isEqualValue(null, undefined)).toBe(false);
  });
});

// validateFieldValue テスト

describe('validateFieldValue', () => {
  const intSchema: ConfigFieldSchema = { key: 'x', label: 'X', type: 'integer', default: 0 };
  const floatSchema: ConfigFieldSchema = { key: 'x', label: 'X', type: 'float', default: 0.0 };
  const strSchema: ConfigFieldSchema = { key: 'x', label: 'X', type: 'string', default: '' };
  const boolSchema: ConfigFieldSchema = { key: 'x', label: 'X', type: 'boolean', default: false };
  const enumSchema: ConfigFieldSchema = { key: 'x', label: 'X', type: 'enum', default: 'a', options: ['a', 'b', 'c'] };

  it('整数スキーマに整数値は通過する', () => {
    expect(validateFieldValue(intSchema, 42)).toBeUndefined();
  });

  it('整数スキーマに浮動小数点はエラーを返す', () => {
    expect(validateFieldValue(intSchema, 3.14)).toBe('整数を入力してください');
  });

  it('整数スキーマに文字列はエラーを返す', () => {
    expect(validateFieldValue(intSchema, 'hello')).toBe('整数を入力してください');
  });

  it('整数スキーマの最小値チェック', () => {
    const schema = { ...intSchema, min: 10 };
    expect(validateFieldValue(schema, 5)).toContain('10');
    expect(validateFieldValue(schema, 10)).toBeUndefined();
  });

  it('整数スキーマの最大値チェック', () => {
    const schema = { ...intSchema, max: 100 };
    expect(validateFieldValue(schema, 101)).toContain('100');
    expect(validateFieldValue(schema, 100)).toBeUndefined();
  });

  it('浮動小数点スキーマに数値は通過する', () => {
    expect(validateFieldValue(floatSchema, 3.14)).toBeUndefined();
  });

  it('浮動小数点スキーマにNaNはエラーを返す', () => {
    expect(validateFieldValue(floatSchema, NaN)).toBe('数値を入力してください');
  });

  it('文字列スキーマに文字列は通過する', () => {
    expect(validateFieldValue(strSchema, 'hello')).toBeUndefined();
  });

  it('文字列スキーマに数値はエラーを返す', () => {
    expect(validateFieldValue(strSchema, 42)).toBe('文字列を入力してください');
  });

  it('文字列スキーマにパターン違反はエラーを返す', () => {
    const schema = { ...strSchema, pattern: '^[a-z]+$' };
    expect(validateFieldValue(schema, 'ABC')).toContain('パターン');
    expect(validateFieldValue(schema, 'abc')).toBeUndefined();
  });

  it('真偽値スキーマに boolean は通過する', () => {
    expect(validateFieldValue(boolSchema, true)).toBeUndefined();
    expect(validateFieldValue(boolSchema, false)).toBeUndefined();
  });

  it('真偽値スキーマに文字列はエラーを返す', () => {
    expect(validateFieldValue(boolSchema, 'yes')).toBe('真偽値を入力してください');
  });

  it('enum スキーマに有効値は通過する', () => {
    expect(validateFieldValue(enumSchema, 'a')).toBeUndefined();
  });

  it('enum スキーマに無効値はエラーを返す', () => {
    expect(validateFieldValue(enumSchema, 'z')).toBe('定義済みの候補を選択してください');
  });
});

// findEntryForField テスト

describe('findEntryForField', () => {
  // ServiceConfigEntryResponse の必須フィールドに合わせて定義する（M-2 監査対応: 型不整合を修正）
  const entry: ServiceConfigEntryResponse = {
    namespace: 'app.general',
    key: 'timeout',
    value: 30,
    version: 1,
  };

  it('最初に一致するネームスペースのエントリを返す', () => {
    const entries = new Map([['app.general::timeout', entry]]);
    const result = findEntryForField(['app.general', 'app.advanced'], 'timeout', entries);
    expect(result).toBe(entry);
  });

  it('存在しないキーは undefined を返す', () => {
    const entries = new Map<string, ServiceConfigEntryResponse>();
    expect(findEntryForField(['app.general'], 'unknown', entries)).toBeUndefined();
  });
});

// updateDirtyField テスト

describe('updateDirtyField', () => {
  // ConfigFieldValue の必須フィールドを全て明示する（M-2 監査対応: 型不整合を修正）
  const baseField: ConfigFieldValue = {
    id: 'app.general::timeout',
    key: 'timeout',
    namespace: 'app.general',
    value: 30,
    originalValue: 30,
    version: 1,
    originalVersion: 1,
    isDirty: false,
  };

  it('元の値と異なる場合は isDirty が true になる', () => {
    const result = updateDirtyField(baseField, 60);
    expect(result.isDirty).toBe(true);
    expect(result.value).toBe(60);
  });

  it('元の値と同じ場合は isDirty が false のまま', () => {
    const result = updateDirtyField(baseField, 30);
    expect(result.isDirty).toBe(false);
  });

  it('エラーが設定される', () => {
    const result = updateDirtyField(baseField, 999, '最大値を超えています');
    expect(result.hasError).toBe('最大値を超えています');
  });
});
