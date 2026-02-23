import { useState, useEffect, useCallback, useMemo } from 'react';
import type { AxiosInstance } from 'axios';
import type { ConfigEditorConfig } from '../types';
import { ConfigInterpreter } from '../ConfigInterpreter';

interface UseConfigEditorOptions {
  client: AxiosInstance;
  serviceName: string;
}

interface UseConfigEditorReturn {
  config: ConfigEditorConfig | null;
  isDirty: boolean;
  hasConflict: boolean;
  updateField: (categoryId: string, key: string, value: unknown) => void;
  save: () => Promise<void>;
  reset: () => void;
  resetFieldToDefault: (categoryId: string, key: string) => void;
}

export function useConfigEditor({ client, serviceName }: UseConfigEditorOptions): UseConfigEditorReturn {
  const [config, setConfig] = useState<ConfigEditorConfig | null>(null);
  const [initialConfig, setInitialConfig] = useState<ConfigEditorConfig | null>(null);
  const [hasConflict, setHasConflict] = useState(false);

  const interpreter = useMemo(() => new ConfigInterpreter(client), [client]);

  useEffect(() => {
    let cancelled = false;
    interpreter.build(serviceName).then((result) => {
      if (!cancelled) {
        setConfig(result);
        setInitialConfig(result);
      }
    });
    return () => { cancelled = true; };
  }, [interpreter, serviceName]);

  const isDirty = (config?.dirtyCount ?? 0) > 0;

  const updateField = useCallback((categoryId: string, key: string, value: unknown) => {
    setConfig((prev) => {
      if (!prev) return prev;

      const categories = prev.categories.map((cat) => {
        if (cat.id !== categoryId) return cat;

        const fieldValues = { ...cat.fieldValues };
        const existing = fieldValues[key];
        if (!existing) return cat;

        const isDirty = value !== existing.originalValue;
        fieldValues[key] = { ...existing, value, isDirty };

        return { ...cat, fieldValues };
      });

      const dirtyCount = categories.reduce((sum, cat) => {
        return sum + Object.values(cat.fieldValues).filter((fv) => fv.isDirty).length;
      }, 0);

      return { ...prev, categories, dirtyCount };
    });
  }, []);

  const save = useCallback(async () => {
    if (!config) return;

    const dirtyFields = config.categories.flatMap((cat) =>
      Object.values(cat.fieldValues).filter((fv) => fv.isDirty),
    );

    try {
      await Promise.all(
        dirtyFields.map((field) =>
          client.put(`/api/v1/config/${field.namespace}/${field.key}`, {
            value: field.value,
          }),
        ),
      );

      setConfig((prev) => {
        if (!prev) return prev;

        const categories = prev.categories.map((cat) => ({
          ...cat,
          fieldValues: Object.fromEntries(
            Object.entries(cat.fieldValues).map(([k, fv]) => [
              k,
              { ...fv, originalValue: fv.value, isDirty: false },
            ]),
          ),
        }));

        return { ...prev, categories, dirtyCount: 0 };
      });
      setHasConflict(false);
    } catch (err: unknown) {
      if (isAxios409(err)) {
        setHasConflict(true);
      } else {
        throw err;
      }
    }
  }, [config, client]);

  const reset = useCallback(() => {
    if (initialConfig) {
      setConfig(initialConfig);
      setHasConflict(false);
    }
  }, [initialConfig]);

  const resetFieldToDefault = useCallback((categoryId: string, key: string) => {
    setConfig((prev) => {
      if (!prev) return prev;

      const categories = prev.categories.map((cat) => {
        if (cat.id !== categoryId) return cat;

        const fieldValues = { ...cat.fieldValues };
        const existing = fieldValues[key];
        if (!existing) return cat;

        const field = cat.fields.find((f) => f.key === key);
        const defaultValue = field?.default;
        const isDirty = defaultValue !== existing.originalValue;
        fieldValues[key] = { ...existing, value: defaultValue, isDirty };

        return { ...cat, fieldValues };
      });

      const dirtyCount = categories.reduce((sum, cat) => {
        return sum + Object.values(cat.fieldValues).filter((fv) => fv.isDirty).length;
      }, 0);

      return { ...prev, categories, dirtyCount };
    });
  }, []);

  return { config, isDirty, hasConflict, updateField, save, reset, resetFieldToDefault };
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
