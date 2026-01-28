/**
 * フィールドタイプリゾルバー
 *
 * ParsedFieldInfo と FieldConfig から最終的なコンポーネント種別を決定
 */

import type { ParsedFieldInfo, FieldConfig, MuiFieldComponent } from '../types.js';

/**
 * フィールドの最終的なコンポーネント種別を決定する
 */
export function resolveFieldComponent(
  parsedInfo: ParsedFieldInfo,
  config?: FieldConfig
): MuiFieldComponent {
  // ユーザー指定があればそれを優先
  if (config?.component) {
    return config.component;
  }

  // カスタムレンダーがあれば custom
  if (config?.render) {
    return 'custom';
  }

  // 配列フィールド
  if (parsedInfo.zodType === 'ZodArray') {
    return 'TextField'; // ArrayField で処理
  }

  // オブジェクトフィールド
  if (parsedInfo.zodType === 'ZodObject') {
    return 'TextField'; // ObjectField で処理
  }

  // TextField の multiline 指定
  if (config?.multiline) {
    return 'TextField';
  }

  // options が指定されていれば Select/RadioGroup
  if (config?.options) {
    return config.options.length <= 4 ? 'RadioGroup' : 'Select';
  }

  // 推論されたコンポーネントを返す
  return parsedInfo.inferredComponent;
}

/**
 * TextField の input type を決定する
 */
export function resolveInputType(
  parsedInfo: ParsedFieldInfo,
  config?: FieldConfig
): string {
  // ユーザー指定があればそれを優先
  if (config?.type) {
    return config.type;
  }

  // 数値型
  if (parsedInfo.zodType === 'ZodNumber') {
    return 'number';
  }

  // メール形式
  if (parsedInfo.isEmail) {
    return 'email';
  }

  // URL 形式
  if (parsedInfo.isUrl) {
    return 'url';
  }

  return 'text';
}

/**
 * フィールドが配列型かどうか
 */
export function isArrayField(parsedInfo: ParsedFieldInfo): boolean {
  return parsedInfo.zodType === 'ZodArray';
}

/**
 * フィールドがオブジェクト型かどうか
 */
export function isObjectField(parsedInfo: ParsedFieldInfo): boolean {
  return parsedInfo.zodType === 'ZodObject';
}

/**
 * フィールドが日付系かどうか
 */
export function isDateField(component: MuiFieldComponent): boolean {
  return ['DatePicker', 'DateTimePicker', 'TimePicker'].includes(component);
}

/**
 * フィールドが選択系かどうか
 */
export function isSelectField(component: MuiFieldComponent): boolean {
  return ['Select', 'RadioGroup', 'Autocomplete'].includes(component);
}

/**
 * フィールドがトグル系かどうか
 */
export function isToggleField(component: MuiFieldComponent): boolean {
  return ['Switch', 'Checkbox'].includes(component);
}
