/**
 * K1s0 Form Generator
 *
 * Zod スキーマから MUI フォームを自動生成するライブラリ
 *
 * @packageDocumentation
 */

// メイン関数
export { createFormFromSchema } from './createFormFromSchema.js';

// コンポーネント
export { FormContainer } from './components/FormContainer.js';
export { FormGrid } from './components/FormGrid.js';
export { FormActions } from './components/FormActions.js';

// フィールドコンポーネント
export {
  MuiTextField,
  MuiSelect,
  MuiRadioGroup,
  MuiCheckbox,
  MuiSwitch,
  MuiDatePicker,
  MuiDateTimePicker,
  MuiTimePicker,
  MuiSlider,
  MuiRating,
  MuiAutocomplete,
  ArrayField,
} from './fields/index.js';

// フック
export { useFormGenerator } from './hooks/useFormGenerator.js';
export { useConditionalField } from './hooks/useConditionalField.js';

// ユーティリティ
export { parseSchema, parseFieldSchema, extractDefaultValues } from './utils/schemaParser.js';
export {
  resolveFieldComponent,
  resolveInputType,
  isArrayField,
  isObjectField,
  isDateField,
  isSelectField,
  isToggleField,
} from './utils/fieldTypeResolver.js';

// 型定義
export type {
  MuiFieldComponent,
  FieldOption,
  FieldConfig,
  ConditionalFieldConfig,
  CustomFieldProps,
  FormGeneratorOptions,
  GeneratedFormProps,
  ParsedFieldInfo,
  FormFieldComponentProps,
  UseFormGeneratorReturn,
} from './types.js';
