import { useState } from 'react';
import { useItems, useDeleteItem, useCategory } from '../../hooks/useDomainMaster';
import { ItemForm } from './ItemForm';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import type { MasterItem } from '../../types/domain-master';

// アイテム一覧コンポーネントのProps
interface ItemListProps {
  categoryCode: string;
}

// アイテム一覧コンポーネント: カテゴリ配下のアイテムをテーブル表示
export function ItemList({ categoryCode }: ItemListProps) {
  // アクティブのみフィルター状態
  const [activeOnly, setActiveOnly] = useState<boolean | undefined>(undefined);
  // フォーム表示状態の管理
  const [editingItem, setEditingItem] = useState<MasterItem | undefined | null>(null);
  // 削除確認ダイアログの対象アイテムコード
  const [deletingItemCode, setDeletingItemCode] = useState<string | null>(null);

  const { data: category } = useCategory(categoryCode);
  const { data: items, isLoading, error } = useItems(categoryCode, activeOnly);
  const deleteItem = useDeleteItem(categoryCode);

  // アイテム削除確認ダイアログを表示
  const handleDelete = (itemCode: string) => {
    setDeletingItemCode(itemCode);
  };

  // 削除確認後に実際の削除を実行
  const handleDeleteConfirm = () => {
    if (deletingItemCode) {
      deleteItem.mutate(deletingItemCode);
    }
    setDeletingItemCode(null);
  };

  if (isLoading) return <div>読み込み中...</div>;
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      {/* 削除確認ダイアログ */}
      <ConfirmDialog
        open={deletingItemCode !== null}
        title="アイテムの削除"
        message={`アイテム "${deletingItemCode}" を削除しますか？`}
        confirmLabel="削除"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeletingItemCode(null)}
      />

      <h1>{category?.display_name ?? categoryCode} - アイテム一覧</h1>

      {/* ナビゲーションリンク */}
      <p>
        <a href="/categories">← カテゴリ一覧に戻る</a>
      </p>

      {/* フィルターとアクションボタン */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label>
          <input
            type="checkbox"
            checked={activeOnly === true}
            onChange={(e) => setActiveOnly(e.target.checked ? true : undefined)}
          />
          アクティブのみ表示
        </label>
        <button onClick={() => setEditingItem(undefined)}>新規作成</button>
      </div>

      {/* アイテム作成・編集フォーム */}
      {editingItem !== null && (
        <ItemForm
          categoryCode={categoryCode}
          item={editingItem}
          items={items ?? []}
          onClose={() => setEditingItem(null)}
        />
      )}

      {/* アイテム一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>コード</th>
            <th style={thStyle}>表示名</th>
            <th style={thStyle}>説明</th>
            <th style={thStyle}>有効期間</th>
            <th style={thStyle}>状態</th>
            <th style={thStyle}>並び順</th>
            <th style={thStyle}>操作</th>
          </tr>
        </thead>
        <tbody>
          {items?.map((item) => (
            <tr key={item.id}>
              <td style={tdStyle}>{item.code}</td>
              <td style={tdStyle}>{item.display_name}</td>
              <td style={tdStyle}>{item.description ?? '-'}</td>
              <td style={tdStyle}>
                {item.effective_from ?? '∞'} ~ {item.effective_until ?? '∞'}
              </td>
              <td style={tdStyle}>{item.is_active ? '有効' : '無効'}</td>
              <td style={tdStyle}>{item.sort_order}</td>
              <td style={tdStyle}>
                <button onClick={() => setEditingItem(item)}>編集</button>
                <a
                  href={`/categories/${categoryCode}/items/${item.code}/versions`}
                  style={{ marginLeft: '4px' }}
                >
                  履歴
                </a>
                <button
                  onClick={() => handleDelete(item.code)}
                  style={{ marginLeft: '4px', color: 'red' }}
                >
                  削除
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {items?.length === 0 && <p>アイテムがありません。</p>}
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
