/**
 * Form Generator 型定義
 *
 * Zod スキーマから MUI フォームを自動生成するための型定義
 */

import type { ReactNode } from 'react';
import type { UseFormReturn, FieldValues, Path, FieldErrors } from 'react-hook-form';
import type { z } from 'zod';

/**
 * MUI コンポーネント種別
 */
export type MuiFieldComponent =
  | 'TextField'
  | 'Select'
  | 'RadioGroup'
  | 'Checkbox'
  | 'Switch'
  | 'DatePicker'
  | 'DateTimePicker'
  | 'TimePicker'
  | 'Slider'
  | 'Autocomplete'
  | 'Rating'
  | 'custom';

/**
 * 選択肢オプション
 */
export interface FieldOption<V = unknown> {
  label: string;
  value: V;
  disabled?: boolean;
}

/**
 * フィールド設定
 */
export interface FieldConfig {
  /** MUI コンポーネント種別 */
  component?: MuiFieldComponent;

  /** Select/RadioGroup 用の選択肢 */
  options?: FieldOption[];

  /** TextField: 複数行 */
  multiline?: boolean;
  /** TextField: 行数 */
  rows?: number;
  /** TextField: 最大行数（自動調整） */
  maxRows?: number;
  /** TextField: 入力タイプ */
  type?: 'text' | 'email' | 'password' | 'tel' | 'url' | 'search';

  /** 数値: 最小値 */
  min?: number;
  /** 数値: 最大値 */
  max?: number;
  /** 数値: ステップ */
  step?: number;

  /** Slider: 目盛り */
  marks?: boolean | { value: number; label: string }[];

  /** 共通: 無効化 */
  disabled?: boolean;
  /** 共通: 読み取り専用 */
  readOnly?: boolean;
  /** 共通: オートフォーカス */
  autoFocus?: boolean;

  /** Grid: このフィールドが占めるカラム数 */
  gridColumn?: number;

  /** カスタムレンダー */
  render?: (props: CustomFieldProps) => ReactNode;

  /** 配列フィールド: 追加ボタンラベル */
  addButtonLabel?: string;
  /** 配列フィールド: 最小アイテム数 */
  minItems?: number;
  /** 配列フィールド: 最大アイテム数 */
  maxItems?: number;
  /** 配列フィールド: アイテムのラベル */
  itemLabels?: Record<string, string>;
}

/**
 * 条件付きフィールド設定
 */
export interface ConditionalFieldConfig<T> {
  /** 対象フィールド名 */
  field: keyof T;
  /** 表示条件 */
  condition: (values: T) => boolean;
}

/**
 * カスタムフィールドに渡される Props
 */
export interface CustomFieldProps<T extends FieldValues = FieldValues> {
  /** フィールド名 */
  name: Path<T>;
  /** ラベル */
  label?: string;
  /** ヘルプテキスト */
  helperText?: string;
  /** エラーメッセージ */
  error?: string;
  /** 無効化状態 */
  disabled?: boolean;
  /** 読み取り専用状態 */
  readOnly?: boolean;
  /** react-hook-form インスタンス */
  form: UseFormReturn<T>;
}

/**
 * Form Generator オプション
 */
export interface FormGeneratorOptions<T extends z.ZodObject<z.ZodRawShape>> {
  /** フィールドラベル */
  labels?: Partial<Record<keyof z.infer<T>, string>>;

  /** プレースホルダー */
  placeholders?: Partial<Record<keyof z.infer<T>, string>>;

  /** ヘルプテキスト */
  helperTexts?: Partial<Record<keyof z.infer<T>, string>>;

  /** フィールド設定 */
  fieldConfig?: Partial<Record<keyof z.infer<T>, FieldConfig>>;

  /** レイアウト方向 */
  layout?: 'vertical' | 'horizontal';

  /** グリッドカラム数 */
  columns?: 1 | 2 | 3 | 4;

  /** グリッド間隔 */
  spacing?: number;

  /** フィールド表示順序 */
  fieldOrder?: (keyof z.infer<T>)[];

  /** 条件付きフィールド */
  conditionalFields?: ConditionalFieldConfig<z.infer<T>>[];

  /** 送信ボタンラベル */
  submitLabel?: string;

  /** キャンセルボタンラベル */
  cancelLabel?: string;

  /** キャンセルボタン表示 */
  showCancel?: boolean;

  /** リセットボタン表示 */
  showReset?: boolean;

  /** MUI TextField variant */
  variant?: 'outlined' | 'filled' | 'standard';

  /** MUI コンポーネントサイズ */
  size?: 'small' | 'medium';

  /** 全幅表示 */
  fullWidth?: boolean;

  /** Boolean フィールドのデフォルトコンポーネント */
  booleanComponent?: 'Switch' | 'Checkbox';

  /** 日付ライブラリの dayjs フォーマット */
  dateFormat?: string;

  /** 日時ライブラリの dayjs フォーマット */
  dateTimeFormat?: string;

  /** 時刻ライブラリの dayjs フォーマット */
  timeFormat?: string;
}

/**
 * 生成されるフォームコンポーネントの Props
 */
export interface GeneratedFormProps<T> {
  /** 初期値 */
  defaultValues?: Partial<T>;

  /** 制御モードの値 */
  values?: T;

  /** 送信時コールバック */
  onSubmit: (values: T) => void | Promise<void>;

  /** キャンセル時コールバック */
  onCancel?: () => void;

  /** 値変更時コールバック */
  onChange?: (values: Partial<T>) => void;

  /** 全体の無効化 */
  disabled?: boolean;

  /** ローディング状態 */
  loading?: boolean;

  /** 読み取り専用 */
  readOnly?: boolean;

  /** 外部から渡す react-hook-form インスタンス */
  form?: UseFormReturn<T>;

  /** 追加の className */
  className?: string;

  /** 追加の sx */
  sx?: Record<string, unknown>;
}

/**
 * 解析されたフィールド情報
 */
export interface ParsedFieldInfo {
  /** フィールド名 */
  name: string;
  /** Zod 型種別 */
  zodType: string;
  /** 必須かどうか */
  required: boolean;
  /** オプショナルかどうか */
  optional: boolean;
  /** デフォルト値 */
  defaultValue?: unknown;
  /** 推論された MUI コンポーネント */
  inferredComponent: MuiFieldComponent;
  /** 最小値（数値/文字列長） */
  min?: number;
  /** 最大値（数値/文字列長） */
  max?: number;
  /** 正規表現パターン */
  pattern?: string;
  /** メール形式 */
  isEmail?: boolean;
  /** URL 形式 */
  isUrl?: boolean;
  /** enum 値 */
  enumValues?: string[];
  /** 配列の要素スキーマ */
  arrayItemSchema?: z.ZodTypeAny;
  /** オブジェクトの内部スキーマ */
  objectSchema?: z.ZodObject<z.ZodRawShape>;
}

/**
 * フォームフィールドコンポーネントの Props
 */
export interface FormFieldComponentProps<T extends FieldValues = FieldValues> {
  /** フィールド名 */
  name: Path<T>;
  /** ラベル */
  label?: string;
  /** プレースホルダー */
  placeholder?: string;
  /** ヘルプテキスト */
  helperText?: string;
  /** フィールド設定 */
  config?: FieldConfig;
  /** パース情報 */
  parsedInfo: ParsedFieldInfo;
  /** react-hook-form インスタンス */
  form: UseFormReturn<T>;
  /** MUI variant */
  variant?: 'outlined' | 'filled' | 'standard';
  /** MUI size */
  size?: 'small' | 'medium';
  /** 全幅 */
  fullWidth?: boolean;
  /** 無効化 */
  disabled?: boolean;
  /** 読み取り専用 */
  readOnly?: boolean;
}

/**
 * useFormGenerator フックの返り値
 */
export interface UseFormGeneratorReturn<T extends FieldValues> {
  /** react-hook-form インスタンス */
  form: UseFormReturn<T>;
  /** 送信ハンドラ */
  handleSubmit: (onSubmit: (values: T) => void | Promise<void>) => (e?: React.BaseSyntheticEvent) => Promise<void>;
  /** リセット関数 */
  reset: () => void;
  /** 送信中かどうか */
  isSubmitting: boolean;
  /** バリデーションエラー */
  errors: FieldErrors<T>;
  /** ダーティ状態 */
  isDirty: boolean;
  /** 有効状態 */
  isValid: boolean;
}
