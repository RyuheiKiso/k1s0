import { useState } from 'react';
import { useCategories, useDeleteCategory } from '../../hooks/useDomainMaster';
import { CategoryForm } from './CategoryForm';
import type { MasterCategory } from '../../types/domain-master';

// カテゴリ一覧コンポーネント: テーブル表示でCRUD操作を提供
export function CategoryList() {
  // アクティブのみ表示フィルターの状態
  const [activeOnly, setActiveOnly] = useState<boolean | undefined>(undefined);
  // フォーム表示状態の管理（null: 非表示, undefined: 新規作成, MasterCategory: 編集）
  const [editingCategory, setEditingCategory] = useState<MasterCategory | undefined | null>(null);

  const { data: categories, isLoading, error } = useCategories(activeOnly);
  const deleteCategory = useDeleteCategory();

  // カテゴリ削除確認ダイアログを表示して削除実行
  const handleDelete = (code: string) => {
    if (window.confirm(`カテゴリ "${code}" を削除しますか？`)) {
      deleteCategory.mutate(code);
    }
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      <h1>マスタカテゴリ一覧</h1>

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
        <button onClick={() => setEditingCategory(undefined)}>新規作成</button>
      </div>

      {/* カテゴリ作成・編集フォーム */}
      {editingCategory !== null && (
        <CategoryForm
          category={editingCategory}
          onClose={() => setEditingCategory(null)}
        />
      )}

      {/* カテゴリ一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>コード</th>
            <th style={thStyle}>表示名</th>
            <th style={thStyle}>説明</th>
            <th style={thStyle}>状態</th>
            <th style={thStyle}>並び順</th>
            <th style={thStyle}>操作</th>
          </tr>
        </thead>
        <tbody>
          {categories?.map((category) => (
            <tr key={category.id}>
              <td style={tdStyle}>
                <a href={`/categories/${category.code}/items`}>{category.code}</a>
              </td>
              <td style={tdStyle}>{category.display_name}</td>
              <td style={tdStyle}>{category.description ?? '-'}</td>
              <td style={tdStyle}>{category.is_active ? '有効' : '無効'}</td>
              <td style={tdStyle}>{category.sort_order}</td>
              <td style={tdStyle}>
                <button onClick={() => setEditingCategory(category)}>編集</button>
                <button
                  onClick={() => handleDelete(category.code)}
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
      {categories?.length === 0 && <p>カテゴリがありません。</p>}
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
