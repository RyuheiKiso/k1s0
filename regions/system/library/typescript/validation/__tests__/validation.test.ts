import { describe, it, expect } from 'vitest';
import {
  validateEmail,
  validateUUID,
  validateURL,
  validateURLNotPrivate,
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
    } catch (e: unknown) {
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
    } catch (e: unknown) {
      expect((e as ValidationError).field).toBe('url');
    }
  });
});

describe('validateURLNotPrivate (M-010)', () => {
  it('パブリックIPアドレスを受け入れる', () => {
    // M-010 監査対応: パブリックIPは通過することを確認する
    expect(() => validateURLNotPrivate('https://example.com')).not.toThrow();
    expect(() => validateURLNotPrivate('https://8.8.8.8')).not.toThrow();
    expect(() => validateURLNotPrivate('http://203.0.113.1')).not.toThrow();
  });

  it('RFC 1918 プライベートIPを拒否する', () => {
    // M-010 監査対応: 10.x.x.x / 172.16-31.x.x / 192.168.x.x を拒否する
    expect(() => validateURLNotPrivate('http://10.0.0.1')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('http://10.255.255.255')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('https://172.16.0.1')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('https://172.31.255.255')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('http://192.168.0.1')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('http://192.168.255.255')).toThrow(ValidationError);
  });

  it('172.15.x.x と 172.32.x.x は拒否しない（RFC 1918 外）', () => {
    // 172.16-31 のみが RFC 1918 対象であることを確認する
    expect(() => validateURLNotPrivate('http://172.15.0.1')).not.toThrow();
    expect(() => validateURLNotPrivate('http://172.32.0.1')).not.toThrow();
  });

  it('loopback アドレスを拒否する', () => {
    // M-010 監査対応: 127.x.x.x を拒否する
    expect(() => validateURLNotPrivate('http://127.0.0.1')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('http://127.255.255.255')).toThrow(ValidationError);
  });

  it('IPv6 loopback を拒否する', () => {
    // M-010 監査対応: ::1 を拒否する
    expect(() => validateURLNotPrivate('http://[::1]')).toThrow(ValidationError);
  });

  it('link-local アドレスを拒否する', () => {
    // M-010 監査対応: 169.254.x.x（APIPA）を拒否する
    expect(() => validateURLNotPrivate('http://169.254.0.1')).toThrow(ValidationError);
    expect(() => validateURLNotPrivate('http://169.254.169.254')).toThrow(ValidationError);
  });

  it('エラーコードが PRIVATE_IP_FORBIDDEN であること', () => {
    // M-010 監査対応: 適切なエラーコードを返すことを確認する
    try {
      validateURLNotPrivate('http://10.0.0.1');
    } catch (e: unknown) {
      expect((e as ValidationError).code).toBe('PRIVATE_IP_FORBIDDEN');
      expect((e as ValidationError).field).toBe('url');
    }
  });

  it('無効な URL に対しては validateURL のエラーを返す', () => {
    // 無効な URL の場合は validateURL の INVALID_URL エラーになることを確認する
    expect(() => validateURLNotPrivate('not-a-url')).toThrow(ValidationError);
    try {
      validateURLNotPrivate('not-a-url');
    } catch (e: unknown) {
      expect((e as ValidationError).code).toBe('INVALID_URL');
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
    } catch (e: unknown) {
      expect((e as ValidationError).code).toBe('INVALID_PAGE');
    }
    try {
      validatePagination(1, 0);
    } catch (e: unknown) {
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
    } catch (e: unknown) {
      expect((e as ValidationError).code).toBe('INVALID_DATE_RANGE');
    }
  });
});

describe('ValidationError code', () => {
  it('emailエラーにcodeが含まれる', () => {
    try {
      validateEmail('bad');
    } catch (e: unknown) {
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
