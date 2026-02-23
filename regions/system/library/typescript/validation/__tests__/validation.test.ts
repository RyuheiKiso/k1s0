import { describe, it, expect } from 'vitest';
import {
  validateEmail,
  validateUUID,
  validateURL,
  validateTenantId,
  ValidationError,
} from '../src/index.js';

describe('validateEmail', () => {
  it('有効なメールアドレスを受け入れる', () => {
    expect(() => validateEmail('user@example.com')).not.toThrow();
  });

  it('無効なメールアドレスでValidationErrorを投げる', () => {
    expect(() => validateEmail('invalid')).toThrow(ValidationError);
    expect(() => validateEmail('invalid')).toThrow('invalid email');
  });

  it('空文字でValidationErrorを投げる', () => {
    expect(() => validateEmail('')).toThrow(ValidationError);
  });
});

describe('validateUUID', () => {
  it('有効なUUIDを受け入れる', () => {
    expect(() => validateUUID('550e8400-e29b-41d4-a716-446655440000')).not.toThrow();
  });

  it('無効なUUIDでValidationErrorを投げる', () => {
    expect(() => validateUUID('not-a-uuid')).toThrow(ValidationError);
    expect(() => validateUUID('not-a-uuid')).toThrow('invalid UUID');
  });

  it('fieldがidであること', () => {
    try {
      validateUUID('bad');
    } catch (e) {
      expect((e as ValidationError).field).toBe('id');
    }
  });
});

describe('validateURL', () => {
  it('有効なURLを受け入れる', () => {
    expect(() => validateURL('https://example.com')).not.toThrow();
    expect(() => validateURL('http://localhost:8080/path')).not.toThrow();
  });

  it('無効なURLでValidationErrorを投げる', () => {
    expect(() => validateURL('not a url')).toThrow(ValidationError);
  });

  it('fieldがurlであること', () => {
    try {
      validateURL('bad');
    } catch (e) {
      expect((e as ValidationError).field).toBe('url');
    }
  });
});

describe('validateTenantId', () => {
  it('有効なテナントIDを受け入れる', () => {
    expect(() => validateTenantId('my-tenant-01')).not.toThrow();
  });

  it('大文字を含むIDでValidationErrorを投げる', () => {
    expect(() => validateTenantId('MyTenant')).toThrow(ValidationError);
  });

  it('短すぎるIDでValidationErrorを投げる', () => {
    expect(() => validateTenantId('ab')).toThrow(ValidationError);
  });
});
