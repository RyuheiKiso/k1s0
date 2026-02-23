import { useState, useEffect } from 'react';
import type { AxiosInstance } from 'axios';
import { useConfigEditor } from './hooks/useConfigEditor';
import { CategoryNav } from './components/CategoryNav';
import { ConfigFieldList } from './components/ConfigFieldList';

interface ConfigEditorPageProps {
  serviceName: string;
  client: AxiosInstance;
}

export function ConfigEditorPage({ serviceName, client }: ConfigEditorPageProps) {
  const [activeCategory, setActiveCategory] = useState('');
  const { config, isDirty, save, reset, hasConflict, updateField, resetFieldToDefault } =
    useConfigEditor({ client, serviceName });

  useEffect(() => {
    if (config && config.categories.length > 0 && !activeCategory) {
      setActiveCategory(config.categories[0].id);
    }
  }, [config, activeCategory]);

  const activeData = config?.categories.find((c) => c.id === activeCategory);

  return (
    <div className="config-editor">
      <div className="config-editor__header">
        <h1>{config?.service} 設定</h1>
        {isDirty && <span>変更 {config?.dirtyCount}件あり</span>}
        <button type="button" onClick={reset} disabled={!isDirty}>
          破棄
        </button>
        <button type="button" onClick={save} disabled={!isDirty}>
          保存
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
          <p>他のユーザーが更新しました。ページを再読み込みしてください。</p>
        </div>
      )}
    </div>
  );
}
