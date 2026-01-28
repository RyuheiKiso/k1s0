/**
 * useFormGenerator - フォームジェネレーター用フック
 */

import { useForm, UseFormReturn, FieldValues, DefaultValues } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useCallback } from 'react';
import type { UseFormGeneratorReturn } from '../types.js';
import { extractDefaultValues } from '../utils/schemaParser.js';

interface UseFormGeneratorOptions<T extends z.ZodObject<z.ZodRawShape>> {
  schema: T;
  defaultValues?: Partial<z.infer<T>>;
  mode?: 'onBlur' | 'onChange' | 'onSubmit' | 'onTouched' | 'all';
}

export function useFormGenerator<T extends z.ZodObject<z.ZodRawShape>>(
  options: UseFormGeneratorOptions<T>
): UseFormGeneratorReturn<z.infer<T>> {
  const { schema, defaultValues, mode = 'onBlur' } = options;

  // スキーマからデフォルト値を抽出
  const schemaDefaults = extractDefaultValues(schema);

  // react-hook-form を初期化
  const form = useForm<z.infer<T>>({
    resolver: zodResolver(schema),
    defaultValues: {
      ...schemaDefaults,
      ...defaultValues,
    } as DefaultValues<z.infer<T>>,
    mode,
  });

  const {
    handleSubmit: rhfHandleSubmit,
    reset: rhfReset,
    formState: { isSubmitting, errors, isDirty, isValid },
  } = form;

  // 送信ハンドラをラップ
  const handleSubmit = useCallback(
    (onSubmit: (values: z.infer<T>) => void | Promise<void>) => {
      return rhfHandleSubmit(onSubmit);
    },
    [rhfHandleSubmit]
  );

  // リセット関数をラップ
  const reset = useCallback(() => {
    rhfReset({
      ...schemaDefaults,
      ...defaultValues,
    } as DefaultValues<z.infer<T>>);
  }, [rhfReset, schemaDefaults, defaultValues]);

  return {
    form: form as UseFormReturn<z.infer<T>>,
    handleSubmit,
    reset,
    isSubmitting,
    errors,
    isDirty,
    isValid,
  };
}
