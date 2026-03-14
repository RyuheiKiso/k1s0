import { useState } from 'react';
import {
  useTenantExtension,
  useUpdateTenantExtension,
  useDeleteTenantExtension,
} from '../../hooks/useDomainMaster';

// テナント拡張フォームのProps
interface TenantExtensionFormProps {
  tenantId: string;
  itemId: string;
}

// テナントマスタ拡張フォーム: テナント固有の表示名・属性のオーバーライドを管理
export function TenantExtensionForm({ tenantId, itemId }: TenantExtensionFormProps) {
  const { data: extension, isLoading, error } = useTenantExtension(tenantId, itemId);
  const updateExtension = useUpdateTenantExtension(tenantId, itemId);
  const deleteExtension = useDeleteTenantExtension(tenantId, itemId);

  // 表示名オーバーライドの入力状態
  const [displayNameOverride, setDisplayNameOverride] = useState('');
  // 属性オーバーライドのJSON入力状態
  const [attributesOverrideJson, setAttributesOverrideJson] = useState('');
  // JSON パースエラー
  const [jsonError, setJsonError] = useState('');

  // 既存データが読み込まれたら入力欄に反映
  useState(() => {
    if (extension) {
      setDisplayNameOverride(extension.display_name_override ?? '');
      setAttributesOverrideJson(
        extension.attributes_override ? JSON.stringify(extension.attributes_override, null, 2) : ''
      );
    }
  });

  // フォーム送信: テナント拡張の保存
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setJsonError('');

    // 属性オーバーライドのJSONパース
    let attributesOverride: Record<string, unknown> | null = null;
    if (attributesOverrideJson.trim()) {
      try {
        attributesOverride = JSON.parse(attributesOverrideJson);
      } catch {
        setJsonError('属性オーバーライドのJSON形式が不正です。');
        return;
      }
    }

    updateExtension.mutate({
      display_name_override: displayNameOverride || null,
      attributes_override: attributesOverride,
    });
  };

  // テナント拡張の削除確認と実行
  const handleDelete = () => {
    if (window.confirm('テナント拡張を削除しますか？')) {
      deleteExtension.mutate();
    }
  };

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px' }}>
      <h2>テナント拡張設定</h2>
      <p style={{ color: '#666', fontSize: '0.9em' }}>
        テナントID: {tenantId} / アイテムID: {itemId}
      </p>

      <form onSubmit={handleSubmit}>
        {/* 表示名オーバーライド入力 */}
        <div style={fieldStyle}>
          <label htmlFor="display-name-override">表示名オーバーライド</label>
          <input
            id="display-name-override"
            value={displayNameOverride}
            onChange={(e) => setDisplayNameOverride(e.target.value)}
            placeholder="空の場合はデフォルト表示名を使用"
          />
        </div>

        {/* 属性オーバーライドのJSON入力 */}
        <div style={fieldStyle}>
          <label htmlFor="attributes-override">属性オーバーライド (JSON)</label>
          <textarea
            id="attributes-override"
            value={attributesOverrideJson}
            onChange={(e) => setAttributesOverrideJson(e.target.value)}
            rows={6}
            placeholder='{"key": "value"}'
            style={{ fontFamily: 'monospace' }}
          />
          {jsonError && <span style={{ color: 'red', fontSize: '0.85em' }}>{jsonError}</span>}
        </div>

        {/* 操作ボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={updateExtension.isPending}>
            保存
          </button>
          {extension && (
            <button
              type="button"
              onClick={handleDelete}
              disabled={deleteExtension.isPending}
              style={{ color: 'red' }}
            >
              削除
            </button>
          )}
        </div>

        {/* APIエラー表示 */}
        {updateExtension.error && (
          <p style={{ color: 'red' }}>
            保存に失敗しました: {(updateExtension.error as Error).message}
          </p>
        )}
        {/* 保存成功メッセージ */}
        {updateExtension.isSuccess && (
          <p style={{ color: 'green' }}>保存しました。</p>
        )}
      </form>
    </div>
  );
}

// フォームフィールドの共通スタイル
const fieldStyle: React.CSSProperties = {
  marginBottom: '12px',
  display: 'flex',
  flexDirection: 'column',
  gap: '4px',
};
