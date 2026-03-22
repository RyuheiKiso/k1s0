import { useState } from 'react';
import { useTenantExtensions, useDeleteTenantExtension } from '../../hooks/useTenantExtensions';
import { TenantExtensionForm } from './TenantExtensionForm';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import type { TenantProjectExtension } from '../../types/projectMaster';

// テナント拡張一覧コンポーネントのProps
interface TenantExtensionListProps {
  tenantId: string;
}

// テナント拡張一覧コンポーネント: テナント固有のステータス定義カスタマイズを一覧表示
export function TenantExtensionList({ tenantId }: TenantExtensionListProps) {
  // フォーム表示状態の管理（null: 非表示, TenantProjectExtension: 編集対象）
  const [editingExtension, setEditingExtension] = useState<TenantProjectExtension | null>(
    null
  );
  // 削除確認ダイアログの対象ステータス定義ID
  const [deletingStatusDefId, setDeletingStatusDefId] = useState<string | null>(null);

  const { data: extensions, isLoading, error } = useTenantExtensions(tenantId);
  const deleteExtension = useDeleteTenantExtension(tenantId, deletingStatusDefId ?? '');

  // 削除確認後に実際の削除を実行
  const handleDeleteConfirm = () => {
    if (deletingStatusDefId) {
      deleteExtension.mutate();
    }
    setDeletingStatusDefId(null);
  };

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      {/* 削除確認ダイアログ */}
      <ConfirmDialog
        open={deletingStatusDefId !== null}
        title="テナント拡張の削除"
        message="このテナント拡張を削除しますか？"
        confirmLabel="削除"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeletingStatusDefId(null)}
      />

      <h2>テナント拡張一覧</h2>
      <p style={{ color: '#666', fontSize: '0.9em' }}>テナントID: {tenantId}</p>

      {/* テナント拡張編集フォーム */}
      {editingExtension !== null && (
        <TenantExtensionForm
          tenantId={tenantId}
          statusDefinitionId={editingExtension.status_definition_id}
          onClose={() => setEditingExtension(null)}
        />
      )}

      {/* テナント拡張一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>ステータス定義ID</th>
            <th style={thStyle}>表示名オーバーライド</th>
            <th style={thStyle}>有効</th>
            <th style={thStyle}>更新日時</th>
            <th style={thStyle}>操作</th>
          </tr>
        </thead>
        <tbody>
          {extensions?.map((ext) => (
            <tr key={ext.id}>
              <td style={tdStyle}>{ext.status_definition_id}</td>
              <td style={tdStyle}>{ext.display_name_override ?? '-'}</td>
              <td style={tdStyle}>{ext.is_enabled ? '○' : '×'}</td>
              <td style={tdStyle}>{new Date(ext.updated_at).toLocaleString('ja-JP')}</td>
              <td style={tdStyle}>
                <button onClick={() => setEditingExtension(ext)}>編集</button>
                <button
                  onClick={() => setDeletingStatusDefId(ext.status_definition_id)}
                  style={{ marginLeft: '4px', color: 'red' }}
                >
                  削除
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {extensions?.length === 0 && <p>テナント拡張がありません。</p>}
    </div>
  );
}

// テーブルヘッダーのスタイル
const thStyle: React.CSSProperties = {
  borderBottom: '2px solid #ccc',
  padding: '8px',
  textAlign: 'left',
};

// テーブルセルのスタイル
const tdStyle: React.CSSProperties = {
  borderBottom: '1px solid #eee',
  padding: '8px',
};
