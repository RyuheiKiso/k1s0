import { useState } from 'react';
import { useProjectTypes, useDeleteProjectType } from '../../hooks/useProjectTypes';
import { ProjectTypeForm } from './ProjectTypeForm';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import type { ProjectType } from '../../types/projectMaster';

// プロジェクトタイプ一覧コンポーネント: テーブル表示でCRUD操作を提供
export function ProjectTypeList() {
  // アクティブのみ表示フィルターの状態
  const [activeOnly, setActiveOnly] = useState<boolean | undefined>(undefined);
  // フォーム表示状態の管理（null: 非表示, undefined: 新規作成, ProjectType: 編集）
  const [editingProjectType, setEditingProjectType] = useState<ProjectType | undefined | null>(null);
  // 削除確認ダイアログの対象プロジェクトタイプID
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const { data: projectTypes, isLoading, error } = useProjectTypes(activeOnly);
  const deleteProjectType = useDeleteProjectType();

  // プロジェクトタイプ削除確認ダイアログを表示
  const handleDelete = (id: string) => {
    setDeletingId(id);
  };

  // 削除確認後に実際の削除を実行
  const handleDeleteConfirm = () => {
    if (deletingId) {
      deleteProjectType.mutate(deletingId);
    }
    setDeletingId(null);
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // 内部エラー詳細をユーザーに直接表示すると情報漏洩になるため汎用メッセージを表示する
  if (error) {
    console.error('ProjectTypeList エラー:', error);
    return <div role="alert">プロジェクトタイプの読み込みに失敗しました。しばらく経ってからもう一度お試しください。</div>;
  }

  return (
    <div>
      {/* 削除確認ダイアログ */}
      <ConfirmDialog
        open={deletingId !== null}
        title="プロジェクトタイプの削除"
        message={`このプロジェクトタイプを削除しますか？`}
        confirmLabel="削除"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeletingId(null)}
      />

      <h1>プロジェクトタイプ一覧</h1>

      {/* フィルターとアクションボタンのツールバー */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label>
          <input
            type="checkbox"
            checked={activeOnly === true}
            onChange={(e) => setActiveOnly(e.target.checked ? true : undefined)}
          />
          アクティブのみ表示
        </label>
        <button onClick={() => setEditingProjectType(undefined)}>新規作成</button>
      </div>

      {/* プロジェクトタイプ作成・編集フォーム */}
      {editingProjectType !== null && (
        <ProjectTypeForm
          projectType={editingProjectType}
          onClose={() => setEditingProjectType(null)}
        />
      )}

      {/* プロジェクトタイプ一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>コード</th>
            <th style={thStyle}>表示名</th>
            <th style={thStyle}>説明</th>
            <th style={thStyle}>デフォルトワークフロー</th>
            <th style={thStyle}>状態</th>
            <th style={thStyle}>並び順</th>
            <th style={thStyle}>操作</th>
          </tr>
        </thead>
        <tbody>
          {projectTypes?.map((projectType) => (
            <tr key={projectType.id}>
              <td style={tdStyle}>
                <a href={`/project-types/${projectType.id}/status-definitions`}>
                  {projectType.code}
                </a>
              </td>
              <td style={tdStyle}>{projectType.display_name}</td>
              <td style={tdStyle}>{projectType.description ?? '-'}</td>
              <td style={tdStyle}>{projectType.default_workflow ?? '-'}</td>
              <td style={tdStyle}>{projectType.is_active ? '有効' : '無効'}</td>
              <td style={tdStyle}>{projectType.sort_order}</td>
              <td style={tdStyle}>
                <button onClick={() => setEditingProjectType(projectType)}>編集</button>
                <button
                  onClick={() => handleDelete(projectType.id)}
                  style={{ marginLeft: '4px', color: 'red' }}
                >
                  削除
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {projectTypes?.length === 0 && <p>プロジェクトタイプがありません。</p>}
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
