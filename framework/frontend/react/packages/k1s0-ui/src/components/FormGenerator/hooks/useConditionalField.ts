/**
 * useConditionalField - 条件付きフィールド表示用フック
 */

import { useMemo } from 'react';
import { UseFormReturn, FieldValues, useWatch } from 'react-hook-form';
import type { ConditionalFieldConfig } from '../types.js';

interface UseConditionalFieldOptions<T extends FieldValues> {
  form: UseFormReturn<T>;
  conditionalFields?: ConditionalFieldConfig<T>[];
}

interface UseConditionalFieldReturn {
  isFieldVisible: (fieldName: string) => boolean;
}

export function useConditionalField<T extends FieldValues>(
  options: UseConditionalFieldOptions<T>
): UseConditionalFieldReturn {
  const { form, conditionalFields = [] } = options;

  // フォームの全値を監視
  const values = useWatch({ control: form.control }) as T;

  // 各フィールドの表示状態を計算
  const visibilityMap = useMemo(() => {
    const map = new Map<string, boolean>();

    for (const config of conditionalFields) {
      const fieldName = String(config.field);
      const isVisible = config.condition(values);
      map.set(fieldName, isVisible);
    }

    return map;
  }, [conditionalFields, values]);

  // フィールドが表示されるかどうかを判定
  const isFieldVisible = (fieldName: string): boolean => {
    // 条件が設定されていなければ常に表示
    if (!visibilityMap.has(fieldName)) {
      return true;
    }
    return visibilityMap.get(fieldName) ?? true;
  };

  return { isFieldVisible };
}
