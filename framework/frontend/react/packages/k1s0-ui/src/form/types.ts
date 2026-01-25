/**
 * フォーム関連の型定義
 */

/**
 * フィールドエラー
 */
export interface FieldError {
  /** フィールド名 */
  field: string;
  /** エラーメッセージ */
  message: string;
  /** エラーコード（オプション） */
  code?: string;
}

/**
 * フォームエラー
 */
export interface FormErrors {
  /** フィールドごとのエラー */
  fields: Record<string, string | undefined>;
  /** フォーム全体のエラー */
  form?: string;
}

/**
 * フォームの状態
 */
export interface FormState<T = Record<string, unknown>> {
  /** フォームの値 */
  values: T;
  /** フォームのエラー */
  errors: FormErrors;
  /** 送信中かどうか */
  isSubmitting: boolean;
  /** 変更されたかどうか */
  isDirty: boolean;
  /** 有効かどうか */
  isValid: boolean;
  /** タッチされたフィールド */
  touched: Record<string, boolean>;
}

/**
 * バリデーションルール
 */
export interface ValidationRule<T = unknown> {
  /** バリデーション関数 */
  validate: (value: T) => boolean;
  /** エラーメッセージ */
  message: string;
}

/**
 * フィールドのバリデーション設定
 */
export interface FieldValidation {
  /** 必須かどうか */
  required?: boolean | string;
  /** 最小文字数 */
  minLength?: number | { value: number; message: string };
  /** 最大文字数 */
  maxLength?: number | { value: number; message: string };
  /** パターン */
  pattern?: RegExp | { value: RegExp; message: string };
  /** カスタムバリデーション */
  validate?: ValidationRule | ValidationRule[];
}

/**
 * フォームフィールドの共通プロパティ
 */
export interface FormFieldBaseProps {
  /** フィールド名 */
  name: string;
  /** ラベル */
  label?: string;
  /** プレースホルダー */
  placeholder?: string;
  /** ヘルプテキスト */
  helperText?: string;
  /** 必須かどうか */
  required?: boolean;
  /** 無効かどうか */
  disabled?: boolean;
  /** 読み取り専用かどうか */
  readOnly?: boolean;
  /** エラーメッセージ */
  error?: string;
  /** バリデーション設定 */
  validation?: FieldValidation;
}
