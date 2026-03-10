import { useEffect, useMemo, useState } from 'react';
import type { AxiosInstance } from 'axios';
import { createApiClient } from '../http/apiClient';
import { useConfigEditor } from './hooks/useConfigEditor';
import { CategoryNav } from './components/CategoryNav';
import { ConfigFieldList } from './components/ConfigFieldList';

interface ConfigEditorPageProps {
  serviceName?: string;
  service?: string;
  client?: AxiosInstance;
  apiBaseURL?: string;
}

export function ConfigEditorPage({
  serviceName,
  service,
  client,
  apiBaseURL = '',
}: ConfigEditorPageProps) {
  const resolvedServiceName = serviceName ?? service ?? '';
  const resolvedClient = useMemo(
    () => client ?? createApiClient({ baseURL: apiBaseURL }),
    [apiBaseURL, client],
  );
  const [activeCategory, setActiveCategory] = useState('');
  const {
    config,
    isLoading,
    error,
    isDirty,
    hasValidationErrors,
    save,
    reset,
    hasConflict,
    updateField,
    resetFieldToDefault,
  } = useConfigEditor({ client: resolvedClient, serviceName: resolvedServiceName });

  useEffect(() => {
    if (config?.categories.length) {
      setActiveCategory((current) => {
        if (current && config.categories.some((category) => category.id === current)) {
          return current;
        }
        return config.categories[0].id;
      });
    }
  }, [config]);

  if (!resolvedServiceName) {
    return <div role="alert">serviceName is required</div>;
  }

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div role="alert">{error}</div>;
  }

  const activeData = config?.categories.find((category) => category.id === activeCategory);

  return (
    <div className="config-editor">
      <div className="config-editor__header">
        <h1>{config?.service ?? resolvedServiceName} Config</h1>
        {isDirty && <span>{config?.dirtyCount} changes pending</span>}
        <button type="button" onClick={reset} disabled={!isDirty}>
          Discard
        </button>
        <button type="button" onClick={save} disabled={!isDirty || hasValidationErrors}>
          Save
        </button>
      </div>
      <div className="config-editor__body">
        <CategoryNav
          categories={config?.categories ?? []}
          activeId={activeCategory}
          onSelect={setActiveCategory}
        />
        {activeData && (
          <ConfigFieldList
            category={activeData}
            onUpdate={(key, value) => updateField(activeCategory, key, value)}
            onResetToDefault={(key) => resetFieldToDefault(activeCategory, key)}
          />
        )}
      </div>
      {hasConflict && (
        <div role="dialog">
          <p>Another user updated this config. Reload and review before saving again.</p>
        </div>
      )}
    </div>
  );
}
