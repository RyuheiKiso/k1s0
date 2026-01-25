/**
 * フォームコンポーネント
 *
 * @packageDocumentation
 */

// 型定義
export type {
  FieldError,
  FormErrors,
  FormState,
  ValidationRule,
  FieldValidation,
  FormFieldBaseProps,
} from './types.js';

// バリデーション
export {
  validateField,
  validateForm,
  hasErrors,
  validationPatterns,
  validationRules,
} from './validation.js';

// フォームフィールド
export {
  FormTextField,
  FormSelect,
  FormCheckbox,
  FormRadioGroup,
  type FormTextFieldProps,
  type FormSelectProps,
  type FormCheckboxProps,
  type FormRadioGroupProps,
} from './FormField.js';

// フォームコンテナ
export {
  FormContainer,
  type FormContainerProps,
  type FormContext,
} from './FormContainer.js';
