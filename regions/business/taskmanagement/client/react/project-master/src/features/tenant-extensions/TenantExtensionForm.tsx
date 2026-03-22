import { useState, useEffect } from 'react';
import {
  useTenantExtension,
  useUpsertTenantExtension,
  useDeleteTenantExtension,
} from '../../hooks/useTenantExtensions';
import { ConfirmDialog } from '../../components/ConfirmDialog';

// テナント拡張フォームのProps
interface TenantExtensionFormProps {
  tenantId: string;
  statusDefinitionId: string;
  onClose?: () => void;
}

// テナントプロジェクト拡張フォーム: テナント固有の表示名・属性のオーバーライドを管理
export function TenantExtensionForm({
  tenantId,
  statusDefinitionId,
  onClose,
}: TenantExtensionFormProps) {
  const { data: extension, isLoading, error } = useTenantExtension(tenantId, statusDefinitionId);
  const upsertExtension = useUpsertTenantExtension(tenantId, statusDefinitionId);
  const deleteExtension = useDeleteTenantExtension(tenantId, statusDefinitionId);

  // 削除確認ダイアログの表示状態
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  // 表示名オーバーライドの入力状態
  const [displayNameOverride, setDisplayNameOverride] = useState('');
  // 属性オーバーライドのJSON入力状態
  const [attributesOverrideJson, setAttributesOverrideJson] = useState('');
  // 有効フラグの入力状態
  const [isEnabled, setIsEnabled] = useState(true);
  // JSON パースエラー
  const [jsonError, setJsonError] = useState('');

  // 既存データが読み込まれたら入力欄に反映（副作用のためuseEffectを使用）
  useEffect(() => {
    if (extension) {
      setDisplayNameOverride(extension.display_name_override ?? '');
      setAttributesOverrideJson(
        extension.attributes_override
          ? JSON.stringify(extension.attributes_override, null, 2)
          : ''
      );
      setIsEnabled(extension.is_enabled);
    }
  }, [extension]);

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

    upsertExtension.mutate(
      {
        display_name_override: displayNameOverride || null,
        attributes_override: attributesOverride,
        is_enabled: isEnabled,
      },
      { onSuccess: () => onClose?.() }
    );
  };

  // テナント拡張の削除確認ダイアログを表示
  const handleDelete = () => {
    setShowDeleteConfirm(true);
  };

  // 削除確認後に実際の削除を実行
  const handleDeleteConfirm = () => {
    setShowDeleteConfirm(false);
    deleteExtension.mutate(undefined, { onSuccess: () => onClose?.() });
  };

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px' }}>
      {/* 削除確認ダイアログ */}
      <ConfirmDialog
        open={showDeleteConfirm}
        title="テナント拡張の削除"
        message="テナント拡張を削除しますか？"
        confirmLabel="削除"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setShowDeleteConfirm(false)}
      />
      <h2>テナント拡張設定</h2>
      <p style={{ color: '#666', fontSize: '0.9em' }}>
        テナントID: {tenantId} / ステータス定義ID: {statusDefinitionId}
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

        {/* 有効フラグ */}
        <div style={fieldStyle}>
          <label>
            <input
              type="checkbox"
              checked={isEnabled}
              onChange={(e) => setIsEnabled(e.target.checked)}
            />
            有効
          </label>
        </div>

        {/* 操作ボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={upsertExtension.isPending}>
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
          {onClose && (
            <button type="button" onClick={onClose}>
              キャンセル
            </button>
          )}
        </div>

        {/* APIエラー表示 */}
        {upsertExtension.error && (
          <p style={{ color: 'red' }}>
            保存に失敗しました: {(upsertExtension.error as Error).message}
          </p>
        )}
        {/* 保存成功メッセージ */}
        {upsertExtension.isSuccess && (
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
