import React, { useCallback, useState, useMemo } from 'react';
import { Box, type BoxProps } from '@mui/material';
import type { FormErrors, FieldValidation } from './types.js';
import { validateForm, hasErrors } from './validation.js';
import { formSpacing } from '../theme/spacing.js';

/**
 * FormContainer のプロパティ
 */
export interface FormContainerProps<T extends Record<string, unknown>>
  extends Omit<BoxProps, 'onSubmit' | 'children'> {
  /** 初期値 */
  initialValues: T;
  /** バリデーション設定 */
  validations?: Partial<Record<keyof T, FieldValidation>>;
  /** 送信時のコールバック */
  onSubmit: (values: T) => void | Promise<void>;
  /** 値変更時のコールバック */
  onValuesChange?: (values: T) => void;
  /** 送信前にバリデーションを行うか */
  validateOnSubmit?: boolean;
  /** 子要素（render props） */
  children: (context: FormContext<T>) => React.ReactNode;
}

/**
 * フォームコンテキスト
 */
export interface FormContext<T extends Record<string, unknown>> {
  /** 現在の値 */
  values: T;
  /** エラー */
  errors: FormErrors;
  /** 送信中かどうか */
  isSubmitting: boolean;
  /** 変更されたかどうか */
  isDirty: boolean;
  /** 有効かどうか */
  isValid: boolean;
  /** 値を設定 */
  setValue: <K extends keyof T>(field: K, value: T[K]) => void;
  /** フィールドのエラーを取得 */
  getFieldError: (field: keyof T) => string | undefined;
  /** フィールドがエラーかどうか */
  hasFieldError: (field: keyof T) => boolean;
  /** フォームを送信 */
  submit: () => Promise<void>;
  /** フォームをリセット */
  reset: () => void;
  /** すべてのフィールドをバリデーション */
  validate: () => boolean;
}

/**
 * フォームコンテナ
 *
 * フォームの状態管理とバリデーションを提供するコンテナコンポーネント。
 * render props パターンでフォームコンテキストを子コンポーネントに渡す。
 *
 * @example
 * ```tsx
 * <FormContainer
 *   initialValues={{ email: '', password: '' }}
 *   validations={{
 *     email: { required: true, pattern: emailPattern },
 *     password: { required: true, minLength: 8 },
 *   }}
 *   onSubmit={async (values) => {
 *     await login(values);
 *   }}
 * >
 *   {({ values, errors, isSubmitting, setValue, submit }) => (
 *     <>
 *       <FormTextField
 *         name="email"
 *         label="メールアドレス"
 *         value={values.email}
 *         error={errors.fields.email}
 *         onChange={(e) => setValue('email', e.target.value)}
 *       />
 *       <FormTextField
 *         name="password"
 *         label="パスワード"
 *         type="password"
 *         value={values.password}
 *         error={errors.fields.password}
 *         onChange={(e) => setValue('password', e.target.value)}
 *       />
 *       <Button onClick={submit} disabled={isSubmitting}>
 *         ログイン
 *       </Button>
 *     </>
 *   )}
 * </FormContainer>
 * ```
 */
export function FormContainer<T extends Record<string, unknown>>({
  initialValues,
  validations = {},
  onSubmit,
  onValuesChange,
  validateOnSubmit = true,
  children,
  ...boxProps
}: FormContainerProps<T>) {
  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<FormErrors>({ fields: {} });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isDirty, setIsDirty] = useState(false);

  const isValid = useMemo(() => !hasErrors(errors), [errors]);

  const setValue = useCallback(
    <K extends keyof T>(field: K, value: T[K]) => {
      setValues((prev) => {
        const next = { ...prev, [field]: value };
        onValuesChange?.(next);
        return next;
      });
      setIsDirty(true);

      // 既にエラーがある場合はリアルタイムでクリア
      if (errors.fields[field as string]) {
        setErrors((prev) => ({
          ...prev,
          fields: { ...prev.fields, [field]: undefined },
        }));
      }
    },
    [errors.fields, onValuesChange]
  );

  const getFieldError = useCallback(
    (field: keyof T): string | undefined => {
      return errors.fields[field as string];
    },
    [errors.fields]
  );

  const hasFieldError = useCallback(
    (field: keyof T): boolean => {
      return Boolean(errors.fields[field as string]);
    },
    [errors.fields]
  );

  const validate = useCallback((): boolean => {
    const newErrors = validateForm(values, validations);
    setErrors(newErrors);
    return !hasErrors(newErrors);
  }, [values, validations]);

  const submit = useCallback(async () => {
    if (validateOnSubmit && !validate()) {
      return;
    }

    setIsSubmitting(true);
    try {
      await onSubmit(values);
    } finally {
      setIsSubmitting(false);
    }
  }, [onSubmit, validate, validateOnSubmit, values]);

  const reset = useCallback(() => {
    setValues(initialValues);
    setErrors({ fields: {} });
    setIsDirty(false);
  }, [initialValues]);

  const context: FormContext<T> = useMemo(
    () => ({
      values,
      errors,
      isSubmitting,
      isDirty,
      isValid,
      setValue,
      getFieldError,
      hasFieldError,
      submit,
      reset,
      validate,
    }),
    [
      values,
      errors,
      isSubmitting,
      isDirty,
      isValid,
      setValue,
      getFieldError,
      hasFieldError,
      submit,
      reset,
      validate,
    ]
  );

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      submit();
    },
    [submit]
  );

  return (
    <Box
      component="form"
      onSubmit={handleSubmit}
      sx={{
        display: 'flex',
        flexDirection: 'column',
        gap: formSpacing.fieldGap,
        ...boxProps.sx,
      }}
      {...boxProps}
    >
      {children(context)}
    </Box>
  );
}
