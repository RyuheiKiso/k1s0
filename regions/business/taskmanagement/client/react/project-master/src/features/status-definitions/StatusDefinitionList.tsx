import { useState } from 'react';
import {
  useStatusDefinitions,
  useDeleteStatusDefinition,
} from '../../hooks/useStatusDefinitions';
import { StatusDefinitionForm } from './StatusDefinitionForm';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import type { StatusDefinition } from '../../types/projectMaster';

// ステータス定義一覧コンポーネントのProps
interface StatusDefinitionListProps {
  projectTypeId: string;
}

// ステータス定義一覧コンポーネント: プロジェクトタイプ配下のステータスをテーブル表示
export function StatusDefinitionList({ projectTypeId }: StatusDefinitionListProps) {
  // フォーム表示状態の管理（null: 非表示, undefined: 新規作成, StatusDefinition: 編集）
  const [editingDef, setEditingDef] = useState<StatusDefinition | undefined | null>(null);
  // 削除確認ダイアログの対象ID
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const { data: statusDefinitions, isLoading, error } = useStatusDefinitions(projectTypeId);
  const deleteStatusDefinition = useDeleteStatusDefinition(projectTypeId);

  // 削除確認ダイアログを表示
  const handleDelete = (id: string) => {
    setDeletingId(id);
  };

  // 削除確認後に実際の削除を実行
  const handleDeleteConfirm = () => {
    if (deletingId) {
      deleteStatusDefinition.mutate(deletingId);
    }
    setDeletingId(null);
  };

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      {/* 削除確認ダイアログ */}
      <ConfirmDialog
        open={deletingId !== null}
        title="ステータス定義の削除"
        message="このステータス定義を削除しますか？"
        confirmLabel="削除"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeletingId(null)}
      />

      <h2>ステータス定義一覧</h2>

      {/* アクションボタン */}
      <div style={{ marginBottom: '16px' }}>
        <button onClick={() => setEditingDef(undefined)}>新規作成</button>
      </div>

      {/* ステータス定義作成・編集フォーム */}
      {editingDef !== null && (
        <StatusDefinitionForm
          projectTypeId={projectTypeId}
          statusDefinition={editingDef}
          onClose={() => setEditingDef(null)}
        />
      )}

      {/* ステータス定義一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>コード</th>
            <th style={thStyle}>表示名</th>
            <th style={thStyle}>色</th>
            <th style={thStyle}>初期状態</th>
            <th style={thStyle}>終了状態</th>
            <th style={thStyle}>並び順</th>
            <th style={thStyle}>操作</th>
          </tr>
        </thead>
        <tbody>
          {statusDefinitions?.map((def) => (
            <tr key={def.id}>
              <td style={tdStyle}>{def.code}</td>
              <td style={tdStyle}>{def.display_name}</td>
              <td style={tdStyle}>
                {def.color ? (
                  <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span
                      style={{
                        width: '16px',
                        height: '16px',
                        borderRadius: '50%',
                        background: def.color,
                        display: 'inline-block',
                      }}
                    />
                    {def.color}
                  </span>
                ) : (
                  '-'
                )}
              </td>
              <td style={tdStyle}>{def.is_initial ? '○' : '-'}</td>
              <td style={tdStyle}>{def.is_terminal ? '○' : '-'}</td>
              <td style={tdStyle}>{def.sort_order}</td>
              <td style={tdStyle}>
                <button onClick={() => setEditingDef(def)}>編集</button>
                <a
                  href={`/status-definitions/${def.id}/versions`}
                  style={{ marginLeft: '4px' }}
                >
                  履歴
                </a>
                <button
                  onClick={() => handleDelete(def.id)}
                  style={{ marginLeft: '4px', color: 'red' }}
                >
                  削除
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {statusDefinitions?.length === 0 && <p>ステータス定義がありません。</p>}
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
