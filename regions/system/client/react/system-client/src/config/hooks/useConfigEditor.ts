import { useCallback, useEffect, useMemo, useState } from 'react';
import type { AxiosInstance } from 'axios';
import type { ConfigEditorConfig } from '../types';
import { ConfigInterpreter } from '../ConfigInterpreter';
import { cloneConfig, updateDirtyField, validateFieldValue } from '../utils';

interface UseConfigEditorOptions {
  client: AxiosInstance;
  serviceName: string;
}

interface UseConfigEditorReturn {
  config: ConfigEditorConfig | null;
  isLoading: boolean;
  error: string | null;
  isDirty: boolean;
  hasValidationErrors: boolean;
  hasConflict: boolean;
  updateField: (categoryId: string, key: string, value: unknown) => void;
  save: () => Promise<void>;
  reset: () => void;
  resetFieldToDefault: (categoryId: string, key: string) => void;
}

export function useConfigEditor({ client, serviceName }: UseConfigEditorOptions): UseConfigEditorReturn {
  const [config, setConfig] = useState<ConfigEditorConfig | null>(null);
  const [initialConfig, setInitialConfig] = useState<ConfigEditorConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasConflict, setHasConflict] = useState(false);

  const interpreter = useMemo(() => new ConfigInterpreter(client), [client]);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    setError(null);

    interpreter.build(serviceName)
      .then((result) => {
        if (!cancelled) {
          setConfig(cloneConfig(result));
          setInitialConfig(cloneConfig(result));
          setHasConflict(false);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to load config');
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [interpreter, serviceName]);

  const isDirty = (config?.dirtyCount ?? 0) > 0;
  const hasValidationErrors = config?.categories.some((category) =>
    Object.values(category.fieldValues).some((field) => Boolean(field.hasError)),
  ) ?? false;

  const updateField = useCallback((categoryId: string, key: string, value: unknown) => {
    setConfig((prev) => {
      if (!prev) return prev;

      const categories = prev.categories.map((category) => {
        if (category.id !== categoryId) return category;

        const existing = category.fieldValues[key];
        const fieldSchema = category.fields.find((field) => field.key === key);
        if (!existing || !fieldSchema) return category;

        return {
          ...category,
          fieldValues: {
            ...category.fieldValues,
            [key]: updateDirtyField(existing, value, validateFieldValue(fieldSchema, value)),
          },
        };
      });

      return {
        ...prev,
        categories,
        dirtyCount: countDirtyFields(categories),
      };
    });
  }, []);

  const save = useCallback(async () => {
    if (!config) return;
    if (hasValidationErrors) {
      throw new Error('Validation errors must be resolved before saving');
    }

    const dirtyFields = config.categories.flatMap((category) =>
      Object.values(category.fieldValues).filter((field) => field.isDirty),
    );

    try {
      const responses = await Promise.all(
        dirtyFields.map((field) =>
          client.put(`/api/v1/config/${encodeURIComponent(field.namespace)}/${encodeURIComponent(field.key)}`, {
            value: field.value,
            version: field.version,
          }),
        ),
      );

      const updatedEntries = new Map<string, { value: unknown; version: number }>();
      for (const response of responses) {
        const updated = response.data as {
          namespace: string;
          key: string;
          value: unknown;
          version: number;
        };
        updatedEntries.set(`${updated.namespace}::${updated.key}`, {
          value: updated.value,
          version: updated.version,
        });
      }

      const nextConfig: ConfigEditorConfig = {
        ...config,
        categories: config.categories.map((category) => ({
          ...category,
          fieldValues: Object.fromEntries(
            Object.entries(category.fieldValues).map(([key, field]) => {
              const updated = updatedEntries.get(field.id);
              const value = updated?.value ?? field.value;
              const version = updated?.version ?? field.version;
              return [key, {
                ...field,
                value,
                originalValue: value,
                version,
                originalVersion: version,
                isDirty: false,
              }];
            }),
          ),
        })),
        dirtyCount: 0,
      };

      setConfig(nextConfig);
      setInitialConfig(cloneConfig(nextConfig));
      setHasConflict(false);
    } catch (err: unknown) {
      if (isAxios409(err)) {
        setHasConflict(true);
      } else {
        throw err;
      }
    }
  }, [client, config, hasValidationErrors]);

  const reset = useCallback(() => {
    if (initialConfig) {
      setConfig(cloneConfig(initialConfig));
      setHasConflict(false);
    }
  }, [initialConfig]);

  const resetFieldToDefault = useCallback((categoryId: string, key: string) => {
    setConfig((prev) => {
      if (!prev) return prev;

      const categories = prev.categories.map((category) => {
        if (category.id !== categoryId) return category;

        const existing = category.fieldValues[key];
        const fieldSchema = category.fields.find((field) => field.key === key);
        if (!existing || !fieldSchema) return category;

        return {
          ...category,
          fieldValues: {
            ...category.fieldValues,
            [key]: updateDirtyField(
              existing,
              fieldSchema.default,
              validateFieldValue(fieldSchema, fieldSchema.default),
            ),
          },
        };
      });

      return {
        ...prev,
        categories,
        dirtyCount: countDirtyFields(categories),
      };
    });
  }, []);

  return {
    config,
    isLoading,
    error,
    isDirty,
    hasValidationErrors,
    hasConflict,
    updateField,
    save,
    reset,
    resetFieldToDefault,
  };
}

function countDirtyFields(config: ConfigEditorConfig['categories']): number {
  return config.reduce((sum, category) => {
    return sum + Object.values(category.fieldValues).filter((field) => field.isDirty).length;
  }, 0);
}

function isAxios409(err: unknown): boolean {
  return (
    typeof err === 'object' &&
    err !== null &&
    'response' in err &&
    typeof (err as { response?: { status?: number } }).response?.status === 'number' &&
    (err as { response: { status: number } }).response.status === 409
  );
}
