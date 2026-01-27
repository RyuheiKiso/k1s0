/**
 * バリデーションユーティリティのテスト
 */

import { describe, it, expect } from 'vitest';
import {
  validateField,
  validateForm,
  hasErrors,
  validationPatterns,
  validationRules,
} from '../../src/form/validation';
import type { FieldValidation, FormErrors } from '../../src/form/types';

describe('validateField', () => {
  describe('required バリデーション', () => {
    it('必須フィールドが空の場合エラーを返すこと', () => {
      const validation: FieldValidation = { required: true };

      expect(validateField('', validation)).toBe('このフィールドは必須です');
      expect(validateField(null, validation)).toBe('このフィールドは必須です');
      expect(validateField(undefined, validation)).toBe('このフィールドは必須です');
      expect(validateField('   ', validation)).toBe('このフィールドは必須です');
    });

    it('カスタムエラーメッセージを使用できること', () => {
      const validation: FieldValidation = { required: 'メールアドレスを入力してください' };

      expect(validateField('', validation)).toBe('メールアドレスを入力してください');
    });

    it('必須フィールドに値がある場合はエラーを返さないこと', () => {
      const validation: FieldValidation = { required: true };

      expect(validateField('value', validation)).toBeUndefined();
    });
  });

  describe('minLength バリデーション', () => {
    it('最小文字数未満の場合エラーを返すこと', () => {
      const validation: FieldValidation = { minLength: 5 };

      expect(validateField('abc', validation)).toBe('5文字以上で入力してください');
    });

    it('カスタムエラーメッセージを使用できること', () => {
      const validation: FieldValidation = {
        minLength: { value: 5, message: 'パスワードは5文字以上必要です' },
      };

      expect(validateField('abc', validation)).toBe('パスワードは5文字以上必要です');
    });

    it('最小文字数以上の場合はエラーを返さないこと', () => {
      const validation: FieldValidation = { minLength: 5 };

      expect(validateField('abcde', validation)).toBeUndefined();
      expect(validateField('abcdef', validation)).toBeUndefined();
    });

    it('空の値は最小文字数チェックをスキップすること', () => {
      const validation: FieldValidation = { minLength: 5 };

      expect(validateField('', validation)).toBeUndefined();
    });
  });

  describe('maxLength バリデーション', () => {
    it('最大文字数を超える場合エラーを返すこと', () => {
      const validation: FieldValidation = { maxLength: 10 };

      expect(validateField('12345678901', validation)).toBe('10文字以内で入力してください');
    });

    it('カスタムエラーメッセージを使用できること', () => {
      const validation: FieldValidation = {
        maxLength: { value: 10, message: '10文字以内にしてください' },
      };

      expect(validateField('12345678901', validation)).toBe('10文字以内にしてください');
    });

    it('最大文字数以下の場合はエラーを返さないこと', () => {
      const validation: FieldValidation = { maxLength: 10 };

      expect(validateField('1234567890', validation)).toBeUndefined();
      expect(validateField('123', validation)).toBeUndefined();
    });
  });

  describe('pattern バリデーション', () => {
    it('パターンに一致しない場合エラーを返すこと', () => {
      const validation: FieldValidation = { pattern: /^[a-z]+$/ };

      expect(validateField('ABC', validation)).toBe('入力形式が正しくありません');
      expect(validateField('123', validation)).toBe('入力形式が正しくありません');
    });

    it('カスタムエラーメッセージを使用できること', () => {
      const validation: FieldValidation = {
        pattern: { value: /^[a-z]+$/, message: '小文字のみ入力できます' },
      };

      expect(validateField('ABC', validation)).toBe('小文字のみ入力できます');
    });

    it('パターンに一致する場合はエラーを返さないこと', () => {
      const validation: FieldValidation = { pattern: /^[a-z]+$/ };

      expect(validateField('abc', validation)).toBeUndefined();
    });
  });

  describe('validate カスタムバリデーション', () => {
    it('カスタムバリデーションが失敗する場合エラーを返すこと', () => {
      const validation: FieldValidation = {
        validate: {
          validate: (value) => (value as string).includes('@'),
          message: 'メールアドレスの形式が正しくありません',
        },
      };

      expect(validateField('test', validation)).toBe('メールアドレスの形式が正しくありません');
    });

    it('カスタムバリデーションが成功する場合はエラーを返さないこと', () => {
      const validation: FieldValidation = {
        validate: {
          validate: (value) => (value as string).includes('@'),
          message: 'エラー',
        },
      };

      expect(validateField('test@example.com', validation)).toBeUndefined();
    });

    it('複数のカスタムバリデーションを配列で指定できること', () => {
      const validation: FieldValidation = {
        validate: [
          { validate: (value) => (value as string).length >= 3, message: '3文字以上必要です' },
          { validate: (value) => (value as string).includes('@'), message: '@が必要です' },
        ],
      };

      expect(validateField('ab', validation)).toBe('3文字以上必要です');
      expect(validateField('abc', validation)).toBe('@が必要です');
      expect(validateField('ab@', validation)).toBe('3文字以上必要です');
      expect(validateField('abc@', validation)).toBeUndefined();
    });
  });

  describe('複合バリデーション', () => {
    it('複数のバリデーションを組み合わせられること', () => {
      const validation: FieldValidation = {
        required: true,
        minLength: 3,
        maxLength: 10,
        pattern: /^[a-z]+$/,
      };

      expect(validateField('', validation)).toBe('このフィールドは必須です');
      expect(validateField('ab', validation)).toBe('3文字以上で入力してください');
      expect(validateField('abcdefghijk', validation)).toBe('10文字以内で入力してください');
      expect(validateField('ABC', validation)).toBe('入力形式が正しくありません');
      expect(validateField('abcdef', validation)).toBeUndefined();
    });
  });
});

describe('validateForm', () => {
  it('複数フィールドのバリデーションを実行できること', () => {
    const values = {
      email: '',
      password: 'ab',
      name: 'John',
    };
    const validations = {
      email: { required: true } as FieldValidation,
      password: { minLength: 8 } as FieldValidation,
      name: { required: true } as FieldValidation,
    };

    const errors = validateForm(values, validations);

    expect(errors.fields.email).toBe('このフィールドは必須です');
    expect(errors.fields.password).toBe('8文字以上で入力してください');
    expect(errors.fields.name).toBeUndefined();
  });

  it('全てのフィールドが有効な場合はエラーがないこと', () => {
    const values = {
      email: 'test@example.com',
      password: '12345678',
    };
    const validations = {
      email: { required: true } as FieldValidation,
      password: { minLength: 8 } as FieldValidation,
    };

    const errors = validateForm(values, validations);

    expect(errors.fields.email).toBeUndefined();
    expect(errors.fields.password).toBeUndefined();
  });
});

describe('hasErrors', () => {
  it('フィールドエラーがある場合 true を返すこと', () => {
    const errors: FormErrors = {
      fields: { email: 'Required' },
    };

    expect(hasErrors(errors)).toBe(true);
  });

  it('フォームエラーがある場合 true を返すこと', () => {
    const errors: FormErrors = {
      fields: {},
      form: 'Form error',
    };

    expect(hasErrors(errors)).toBe(true);
  });

  it('エラーがない場合 false を返すこと', () => {
    const errors: FormErrors = {
      fields: {},
    };

    expect(hasErrors(errors)).toBe(false);
  });

  it('undefined のフィールドエラーは無視されること', () => {
    const errors: FormErrors = {
      fields: { email: undefined, password: undefined },
    };

    expect(hasErrors(errors)).toBe(false);
  });
});

describe('validationPatterns', () => {
  describe('email パターン', () => {
    it('有効なメールアドレスを許可すること', () => {
      expect(validationPatterns.email.test('test@example.com')).toBe(true);
      expect(validationPatterns.email.test('user.name@domain.co.jp')).toBe(true);
      expect(validationPatterns.email.test('user+tag@example.org')).toBe(true);
    });

    it('無効なメールアドレスを拒否すること', () => {
      expect(validationPatterns.email.test('invalid')).toBe(false);
      expect(validationPatterns.email.test('@example.com')).toBe(false);
      expect(validationPatterns.email.test('test@')).toBe(false);
    });
  });

  describe('phone パターン', () => {
    it('有効な電話番号を許可すること', () => {
      expect(validationPatterns.phone.test('0312345678')).toBe(true);
      expect(validationPatterns.phone.test('09012345678')).toBe(true);
    });

    it('無効な電話番号を拒否すること', () => {
      expect(validationPatterns.phone.test('1234567890')).toBe(false);
      expect(validationPatterns.phone.test('03-1234-5678')).toBe(false);
    });
  });

  describe('postalCode パターン', () => {
    it('有効な郵便番号を許可すること', () => {
      expect(validationPatterns.postalCode.test('123-4567')).toBe(true);
      expect(validationPatterns.postalCode.test('1234567')).toBe(true);
    });

    it('無効な郵便番号を拒否すること', () => {
      expect(validationPatterns.postalCode.test('12-3456')).toBe(false);
      expect(validationPatterns.postalCode.test('1234-567')).toBe(false);
    });
  });

  describe('url パターン', () => {
    it('有効な URL を許可すること', () => {
      expect(validationPatterns.url.test('https://example.com')).toBe(true);
      expect(validationPatterns.url.test('http://localhost:3000')).toBe(true);
    });

    it('無効な URL を拒否すること', () => {
      expect(validationPatterns.url.test('example.com')).toBe(false);
      expect(validationPatterns.url.test('ftp://example.com')).toBe(false);
    });
  });

  describe('alphanumeric パターン', () => {
    it('半角英数字を許可すること', () => {
      expect(validationPatterns.alphanumeric.test('abc123')).toBe(true);
      expect(validationPatterns.alphanumeric.test('ABC')).toBe(true);
    });

    it('記号を含む文字列を拒否すること', () => {
      expect(validationPatterns.alphanumeric.test('abc-123')).toBe(false);
      expect(validationPatterns.alphanumeric.test('abc_123')).toBe(false);
    });
  });

  describe('slug パターン', () => {
    it('スラグ形式を許可すること', () => {
      expect(validationPatterns.slug.test('my-slug')).toBe(true);
      expect(validationPatterns.slug.test('my_slug_123')).toBe(true);
    });

    it('無効なスラグを拒否すること', () => {
      expect(validationPatterns.slug.test('my slug')).toBe(false);
      expect(validationPatterns.slug.test('my.slug')).toBe(false);
    });
  });
});

describe('validationRules', () => {
  it('email ルールが正しく設定されていること', () => {
    expect(validationRules.email.pattern.value).toBe(validationPatterns.email);
    expect(validationRules.email.pattern.message).toBeDefined();
  });

  it('phone ルールが正しく設定されていること', () => {
    expect(validationRules.phone.pattern.value).toBe(validationPatterns.phone);
  });

  it('postalCode ルールが正しく設定されていること', () => {
    expect(validationRules.postalCode.pattern.value).toBe(validationPatterns.postalCode);
  });

  it('url ルールが正しく設定されていること', () => {
    expect(validationRules.url.pattern.value).toBe(validationPatterns.url);
  });
});
