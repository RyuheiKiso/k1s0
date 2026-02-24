import { describe, it, expect } from 'vitest';
import {
  validateEmail,
  validateUUID,
  validateURL,
  validateTenantId,
  validatePagination,
  validateDateRange,
  ValidationError,
  ValidationErrors,
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

describe('validatePagination', () => {
  it('有効なページネーションを受け入れる', () => {
    expect(() => validatePagination(1, 10)).not.toThrow();
    expect(() => validatePagination(1, 1)).not.toThrow();
    expect(() => validatePagination(1, 100)).not.toThrow();
    expect(() => validatePagination(999, 50)).not.toThrow();
  });

  it('page < 1 でValidationErrorを投げる', () => {
    expect(() => validatePagination(0, 10)).toThrow(ValidationError);
    expect(() => validatePagination(-1, 10)).toThrow(ValidationError);
  });

  it('perPage が範囲外でValidationErrorを投げる', () => {
    expect(() => validatePagination(1, 0)).toThrow(ValidationError);
    expect(() => validatePagination(1, 101)).toThrow(ValidationError);
  });

  it('エラーコードが正しいこと', () => {
    try {
      validatePagination(0, 10);
    } catch (e) {
      expect((e as ValidationError).code).toBe('INVALID_PAGE');
    }
    try {
      validatePagination(1, 0);
    } catch (e) {
      expect((e as ValidationError).code).toBe('INVALID_PER_PAGE');
    }
  });
});

describe('validateDateRange', () => {
  it('有効な日付範囲を受け入れる', () => {
    const start = new Date('2024-01-01');
    const end = new Date('2024-12-31');
    expect(() => validateDateRange(start, end)).not.toThrow();
  });

  it('同一日付を受け入れる', () => {
    const dt = new Date('2024-06-15');
    expect(() => validateDateRange(dt, dt)).not.toThrow();
  });

  it('開始日が終了日より後の場合にValidationErrorを投げる', () => {
    const start = new Date('2024-12-31');
    const end = new Date('2024-01-01');
    expect(() => validateDateRange(start, end)).toThrow(ValidationError);
  });

  it('エラーコードが正しいこと', () => {
    try {
      validateDateRange(new Date('2024-12-31'), new Date('2024-01-01'));
    } catch (e) {
      expect((e as ValidationError).code).toBe('INVALID_DATE_RANGE');
    }
  });
});

describe('ValidationError code', () => {
  it('emailエラーにcodeが含まれる', () => {
    try {
      validateEmail('bad');
    } catch (e) {
      expect((e as ValidationError).code).toBe('INVALID_EMAIL');
    }
  });
});

describe('ValidationErrors', () => {
  it('空のコレクションはhasErrors()がfalse', () => {
    const errors = new ValidationErrors();
    expect(errors.hasErrors()).toBe(false);
    expect(errors.getErrors()).toHaveLength(0);
  });

  it('エラー追加後にhasErrors()がtrue', () => {
    const errors = new ValidationErrors();
    errors.add(new ValidationError('email', 'bad', 'INVALID_EMAIL'));
    errors.add(new ValidationError('page', 'bad', 'INVALID_PAGE'));

    expect(errors.hasErrors()).toBe(true);
    expect(errors.getErrors()).toHaveLength(2);
    expect(errors.getErrors()[0].code).toBe('INVALID_EMAIL');
    expect(errors.getErrors()[1].code).toBe('INVALID_PAGE');
  });
});
