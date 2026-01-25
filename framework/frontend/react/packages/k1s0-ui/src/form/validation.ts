/**
 * フォームバリデーションユーティリティ
 */

import type { FieldValidation, FormErrors } from './types.js';

/**
 * 単一フィールドのバリデーション
 */
export function validateField(
  value: unknown,
  validation: FieldValidation
): string | undefined {
  const stringValue = String(value ?? '');

  // 必須チェック
  if (validation.required) {
    const isEmpty = value === undefined || value === null || stringValue.trim() === '';
    if (isEmpty) {
      return typeof validation.required === 'string'
        ? validation.required
        : 'このフィールドは必須です';
    }
  }

  // 値が空の場合は以降のバリデーションをスキップ
  if (!stringValue) {
    return undefined;
  }

  // 最小文字数
  if (validation.minLength) {
    const { value: min, message } =
      typeof validation.minLength === 'number'
        ? { value: validation.minLength, message: `${validation.minLength}文字以上で入力してください` }
        : validation.minLength;

    if (stringValue.length < min) {
      return message;
    }
  }

  // 最大文字数
  if (validation.maxLength) {
    const { value: max, message } =
      typeof validation.maxLength === 'number'
        ? { value: validation.maxLength, message: `${validation.maxLength}文字以内で入力してください` }
        : validation.maxLength;

    if (stringValue.length > max) {
      return message;
    }
  }

  // パターン
  if (validation.pattern) {
    const { value: pattern, message } =
      validation.pattern instanceof RegExp
        ? { value: validation.pattern, message: '入力形式が正しくありません' }
        : validation.pattern;

    if (!pattern.test(stringValue)) {
      return message;
    }
  }

  // カスタムバリデーション
  if (validation.validate) {
    const rules = Array.isArray(validation.validate)
      ? validation.validate
      : [validation.validate];

    for (const rule of rules) {
      if (!rule.validate(value)) {
        return rule.message;
      }
    }
  }

  return undefined;
}

/**
 * フォーム全体のバリデーション
 */
export function validateForm<T extends Record<string, unknown>>(
  values: T,
  validations: Partial<Record<keyof T, FieldValidation>>
): FormErrors {
  const errors: FormErrors = { fields: {} };

  for (const [field, validation] of Object.entries(validations)) {
    if (validation) {
      const value = values[field];
      const error = validateField(value, validation as FieldValidation);
      if (error) {
        errors.fields[field] = error;
      }
    }
  }

  return errors;
}

/**
 * フォームエラーが存在するかチェック
 */
export function hasErrors(errors: FormErrors): boolean {
  return Object.values(errors.fields).some(Boolean) || Boolean(errors.form);
}

/**
 * よく使うバリデーションパターン
 */
export const validationPatterns = {
  /** メールアドレス */
  email: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/,
  /** 電話番号（日本形式） */
  phone: /^0\d{9,10}$/,
  /** 郵便番号（日本形式） */
  postalCode: /^\d{3}-?\d{4}$/,
  /** URL */
  url: /^https?:\/\/.+/,
  /** 半角英数字 */
  alphanumeric: /^[a-zA-Z0-9]+$/,
  /** 半角英数字とハイフン・アンダースコア */
  slug: /^[a-zA-Z0-9_-]+$/,
} as const;

/**
 * よく使うバリデーションルール
 */
export const validationRules = {
  /** メールアドレス */
  email: {
    pattern: {
      value: validationPatterns.email,
      message: '有効なメールアドレスを入力してください',
    },
  },

  /** 電話番号 */
  phone: {
    pattern: {
      value: validationPatterns.phone,
      message: '有効な電話番号を入力してください（例: 0312345678）',
    },
  },

  /** 郵便番号 */
  postalCode: {
    pattern: {
      value: validationPatterns.postalCode,
      message: '有効な郵便番号を入力してください（例: 123-4567）',
    },
  },

  /** URL */
  url: {
    pattern: {
      value: validationPatterns.url,
      message: '有効なURLを入力してください',
    },
  },
} as const;
