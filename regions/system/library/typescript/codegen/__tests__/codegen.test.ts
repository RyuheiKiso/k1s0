import { describe, it, expect } from 'vitest';
import {
  validateConfig,
  hasGrpc,
  hasRest,
  hasDatabase,
  toSnakeCase,
  toPascalCase,
  toKebabCase,
  toCamelCase,
  CodegenError,
  type ScaffoldConfig,
  type GenerateResult,
  type ValidationResult,
} from '../src/index.js';

describe('validateConfig', () => {
  // 有効な設定でバリデーションが成功することを確認する
  it('有効な設定で valid: true を返す', () => {
    const config: ScaffoldConfig = {
      name: 'auth',
      tier: 'system',
      apiStyle: 'rest',
      database: 'postgres',
      description: '認証サーバー',
      generateClient: true,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  // 名前が空の場合にエラーを返すことを確認する
  it('名前が空の場合にエラーを返す', () => {
    const config: ScaffoldConfig = {
      name: '',
      tier: 'system',
      apiStyle: 'rest',
      database: 'none',
      description: 'テスト',
      generateClient: false,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain('name is required');
  });

  // 名前が不正な形式の場合にエラーを返すことを確認する
  it('名前が大文字で始まる場合にエラーを返す', () => {
    const config: ScaffoldConfig = {
      name: 'Auth',
      tier: 'system',
      apiStyle: 'rest',
      database: 'none',
      description: 'テスト',
      generateClient: false,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThan(0);
  });

  // 説明が空の場合にエラーを返すことを確認する
  it('説明が空の場合にエラーを返す', () => {
    const config: ScaffoldConfig = {
      name: 'auth',
      tier: 'system',
      apiStyle: 'rest',
      database: 'none',
      description: '',
      generateClient: false,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain('description is required');
  });

  // gRPC 使用時に protoPath がない場合にエラーを返すことを確認する
  it('gRPC 使用時に protoPath がない場合にエラーを返す', () => {
    const config: ScaffoldConfig = {
      name: 'auth',
      tier: 'system',
      apiStyle: 'grpc',
      database: 'none',
      description: 'テスト',
      generateClient: false,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain(
      'protoPath is required when apiStyle includes grpc',
    );
  });

  // both スタイルで protoPath がない場合にエラーを返すことを確認する
  it('both スタイルで protoPath がない場合にエラーを返す', () => {
    const config: ScaffoldConfig = {
      name: 'auth',
      tier: 'system',
      apiStyle: 'both',
      database: 'postgres',
      description: 'テスト',
      generateClient: true,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain(
      'protoPath is required when apiStyle includes grpc',
    );
  });

  // gRPC 使用時に protoPath があれば成功することを確認する
  it('gRPC 使用時に protoPath があれば valid を返す', () => {
    const config: ScaffoldConfig = {
      name: 'auth',
      tier: 'system',
      apiStyle: 'grpc',
      database: 'none',
      description: 'テスト',
      protoPath: 'proto/auth.proto',
      generateClient: false,
    };
    const result = validateConfig(config);
    expect(result.valid).toBe(true);
  });
});

describe('hasGrpc', () => {
  // grpc と both で true を返すことを確認する
  it('grpc で true を返す', () => {
    expect(hasGrpc('grpc')).toBe(true);
  });

  it('both で true を返す', () => {
    expect(hasGrpc('both')).toBe(true);
  });

  it('rest で false を返す', () => {
    expect(hasGrpc('rest')).toBe(false);
  });
});

describe('hasRest', () => {
  // rest と both で true を返すことを確認する
  it('rest で true を返す', () => {
    expect(hasRest('rest')).toBe(true);
  });

  it('both で true を返す', () => {
    expect(hasRest('both')).toBe(true);
  });

  it('grpc で false を返す', () => {
    expect(hasRest('grpc')).toBe(false);
  });
});

describe('hasDatabase', () => {
  // postgres で true、none で false を返すことを確認する
  it('postgres で true を返す', () => {
    expect(hasDatabase('postgres')).toBe(true);
  });

  it('none で false を返す', () => {
    expect(hasDatabase('none')).toBe(false);
  });
});

describe('toSnakeCase', () => {
  // 各種ケースからスネークケースへの変換を確認する
  it('キャメルケースをスネークケースに変換する', () => {
    expect(toSnakeCase('fooBar')).toBe('foo_bar');
  });

  it('パスカルケースをスネークケースに変換する', () => {
    expect(toSnakeCase('FooBar')).toBe('foo_bar');
  });

  it('ケバブケースをスネークケースに変換する', () => {
    expect(toSnakeCase('foo-bar')).toBe('foo_bar');
  });

  it('既にスネークケースの場合はそのまま返す', () => {
    expect(toSnakeCase('foo_bar')).toBe('foo_bar');
  });
});

describe('toPascalCase', () => {
  // 各種ケースからパスカルケースへの変換を確認する
  it('スネークケースをパスカルケースに変換する', () => {
    expect(toPascalCase('foo_bar')).toBe('FooBar');
  });

  it('ケバブケースをパスカルケースに変換する', () => {
    expect(toPascalCase('foo-bar')).toBe('FooBar');
  });

  it('キャメルケースをパスカルケースに変換する', () => {
    expect(toPascalCase('fooBar')).toBe('FooBar');
  });
});

describe('toKebabCase', () => {
  // 各種ケースからケバブケースへの変換を確認する
  it('キャメルケースをケバブケースに変換する', () => {
    expect(toKebabCase('fooBar')).toBe('foo-bar');
  });

  it('パスカルケースをケバブケースに変換する', () => {
    expect(toKebabCase('FooBar')).toBe('foo-bar');
  });

  it('スネークケースをケバブケースに変換する', () => {
    expect(toKebabCase('foo_bar')).toBe('foo-bar');
  });
});

describe('toCamelCase', () => {
  // 各種ケースからキャメルケースへの変換を確認する
  it('スネークケースをキャメルケースに変換する', () => {
    expect(toCamelCase('foo_bar')).toBe('fooBar');
  });

  it('ケバブケースをキャメルケースに変換する', () => {
    expect(toCamelCase('foo-bar')).toBe('fooBar');
  });

  it('パスカルケースをキャメルケースに変換する', () => {
    expect(toCamelCase('FooBar')).toBe('fooBar');
  });
});

describe('CodegenError', () => {
  // CodegenError のプロパティが正しく設定されることを確認する
  it('code と message が正しく設定される', () => {
    const err = new CodegenError('INVALID_CONFIG', 'config is invalid');
    expect(err.code).toBe('INVALID_CONFIG');
    expect(err.message).toBe('config is invalid');
    expect(err.name).toBe('CodegenError');
  });

  // Error を継承していることを確認する
  it('Error を継承している', () => {
    const err = new CodegenError('TEST', 'test error');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(CodegenError);
  });
});

describe('GenerateResult', () => {
  // GenerateResult の型が正しく使えることを確認する
  it('created と skipped を持つオブジェクトを作成できる', () => {
    const result: GenerateResult = {
      created: ['src/main.ts', 'src/config.ts'],
      skipped: ['src/index.ts'],
    };
    expect(result.created).toHaveLength(2);
    expect(result.skipped).toHaveLength(1);
  });
});

describe('ValidationResult', () => {
  // ValidationResult の型が正しく使えることを確認する
  it('valid と errors を持つオブジェクトを作成できる', () => {
    const result: ValidationResult = {
      valid: false,
      errors: ['name is required'],
    };
    expect(result.valid).toBe(false);
    expect(result.errors).toContain('name is required');
  });
});
