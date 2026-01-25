import React, { useCallback, useState } from 'react';
import {
  TextField,
  type TextFieldProps,
  FormControl,
  FormHelperText,
  InputLabel,
  Select,
  MenuItem,
  type SelectProps,
  type SelectChangeEvent,
  Checkbox,
  FormControlLabel,
  type CheckboxProps,
  RadioGroup,
  Radio,
  type RadioGroupProps,
} from '@mui/material';
import type { FormFieldBaseProps, FieldValidation } from './types.js';
import { validateField } from './validation.js';

/**
 * FormTextField のプロパティ
 */
export interface FormTextFieldProps
  extends Omit<TextFieldProps, 'name' | 'error' | 'label' | 'helperText' | 'required' | 'disabled'>,
    FormFieldBaseProps {
  /** 値変更時のコールバック */
  onValueChange?: (value: string) => void;
}

/**
 * バリデーション連携付きテキストフィールド
 *
 * @example
 * ```tsx
 * <FormTextField
 *   name="email"
 *   label="メールアドレス"
 *   required
 *   validation={{
 *     required: 'メールアドレスを入力してください',
 *     pattern: {
 *       value: /^[^@]+@[^@]+\.[^@]+$/,
 *       message: '有効なメールアドレスを入力してください'
 *     }
 *   }}
 * />
 * ```
 */
export function FormTextField({
  name,
  label,
  helperText,
  error,
  validation,
  required,
  onValueChange,
  onBlur,
  ...props
}: FormTextFieldProps) {
  const [localError, setLocalError] = useState<string | undefined>(undefined);
  const [touched, setTouched] = useState(false);

  const displayError = error ?? (touched ? localError : undefined);
  const effectiveValidation: FieldValidation = {
    ...validation,
    required: required ?? validation?.required,
  };

  const handleBlur = useCallback(
    (e: React.FocusEvent<HTMLInputElement>) => {
      setTouched(true);
      if (effectiveValidation) {
        const error = validateField(e.target.value, effectiveValidation);
        setLocalError(error);
      }
      onBlur?.(e);
    },
    [effectiveValidation, onBlur]
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      onValueChange?.(value);

      // 既にエラーが表示されている場合はリアルタイムでバリデーション
      if (touched && effectiveValidation) {
        const error = validateField(value, effectiveValidation);
        setLocalError(error);
      }

      props.onChange?.(e);
    },
    [effectiveValidation, onValueChange, props, touched]
  );

  return (
    <TextField
      {...props}
      name={name}
      label={label}
      required={Boolean(required ?? validation?.required)}
      error={Boolean(displayError)}
      helperText={displayError ?? helperText}
      onBlur={handleBlur}
      onChange={handleChange}
    />
  );
}

/**
 * FormSelect のプロパティ
 */
export interface FormSelectProps
  extends Omit<SelectProps, 'name' | 'error' | 'label' | 'required' | 'disabled'>,
    Omit<FormFieldBaseProps, 'placeholder'> {
  /** 選択肢 */
  options: Array<{ value: string | number; label: string }>;
  /** 値変更時のコールバック */
  onValueChange?: (value: string | number) => void;
}

/**
 * バリデーション連携付きセレクト
 */
export function FormSelect({
  name,
  label,
  helperText,
  error,
  validation,
  required,
  options,
  onValueChange,
  onChange,
  value,
  fullWidth,
  size,
  disabled,
  ...props
}: FormSelectProps) {
  const [localError, setLocalError] = useState<string | undefined>(undefined);
  const [touched, setTouched] = useState(false);

  const displayError = error ?? (touched ? localError : undefined);
  const effectiveValidation: FieldValidation = {
    ...validation,
    required: required ?? validation?.required,
  };

  const handleBlur = useCallback(() => {
    setTouched(true);
    if (effectiveValidation) {
      const validationError = validateField(value, effectiveValidation);
      setLocalError(validationError);
    }
  }, [effectiveValidation, value]);

  const handleChange = useCallback(
    (event: SelectChangeEvent<unknown>, child: React.ReactNode) => {
      const newValue = event.target.value;
      onValueChange?.(newValue as string | number);
      onChange?.(event, child);
    },
    [onValueChange, onChange]
  );

  const labelId = `${name}-label`;

  return (
    <FormControl
      error={Boolean(displayError)}
      required={Boolean(required ?? validation?.required)}
      fullWidth={fullWidth}
      size={size}
      disabled={disabled}
    >
      {label && <InputLabel id={labelId}>{label}</InputLabel>}
      <Select
        {...props}
        name={name}
        value={value}
        labelId={labelId}
        label={label}
        onBlur={handleBlur}
        onChange={handleChange}
      >
        {options.map((option) => (
          <MenuItem key={option.value} value={option.value}>
            {option.label}
          </MenuItem>
        ))}
      </Select>
      {(displayError || helperText) && (
        <FormHelperText>{displayError ?? helperText}</FormHelperText>
      )}
    </FormControl>
  );
}

/**
 * FormCheckbox のプロパティ
 */
export interface FormCheckboxProps
  extends Omit<CheckboxProps, 'name' | 'disabled'>,
    Omit<FormFieldBaseProps, 'placeholder' | 'required'> {
  /** 値変更時のコールバック */
  onValueChange?: (checked: boolean) => void;
}

/**
 * バリデーション連携付きチェックボックス
 */
export function FormCheckbox({
  name,
  label,
  helperText,
  error,
  disabled,
  onValueChange,
  onChange,
  ...props
}: FormCheckboxProps) {
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
      onValueChange?.(checked);
      onChange?.(e, checked);
    },
    [onValueChange, onChange]
  );

  return (
    <FormControl error={Boolean(error)} disabled={disabled}>
      <FormControlLabel
        control={<Checkbox {...props} name={name} onChange={handleChange} />}
        label={label ?? ''}
      />
      {(error || helperText) && (
        <FormHelperText>{error ?? helperText}</FormHelperText>
      )}
    </FormControl>
  );
}

/**
 * FormRadioGroup のプロパティ
 */
export interface FormRadioGroupProps
  extends Omit<RadioGroupProps, 'name'>,
    FormFieldBaseProps {
  /** 選択肢 */
  options: Array<{ value: string; label: string }>;
  /** 値変更時のコールバック */
  onValueChange?: (value: string) => void;
  /** 横並びにするか */
  row?: boolean;
}

/**
 * バリデーション連携付きラジオグループ
 */
export function FormRadioGroup({
  name,
  label,
  helperText,
  error,
  validation,
  required,
  options,
  onValueChange,
  row = false,
  disabled,
  ...props
}: FormRadioGroupProps) {
  const [localError, setLocalError] = useState<string | undefined>(undefined);
  const [touched, setTouched] = useState(false);

  const displayError = error ?? (touched ? localError : undefined);
  const effectiveValidation: FieldValidation = {
    ...validation,
    required: required ?? validation?.required,
  };

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      setTouched(true);
      onValueChange?.(value);

      if (effectiveValidation) {
        const error = validateField(value, effectiveValidation);
        setLocalError(error);
      }

      props.onChange?.(e, value);
    },
    [effectiveValidation, onValueChange, props]
  );

  return (
    <FormControl
      error={Boolean(displayError)}
      required={Boolean(required ?? validation?.required)}
      disabled={disabled}
    >
      {label && <InputLabel shrink>{label}</InputLabel>}
      <RadioGroup {...props} name={name} row={row} onChange={handleChange}>
        {options.map((option) => (
          <FormControlLabel
            key={option.value}
            value={option.value}
            control={<Radio />}
            label={option.label}
          />
        ))}
      </RadioGroup>
      {(displayError || helperText) && (
        <FormHelperText>{displayError ?? helperText}</FormHelperText>
      )}
    </FormControl>
  );
}
