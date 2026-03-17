import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useInventoryList } from '../../hooks/useInventory';
import type { InventoryStatus } from '../../types/inventory';
import styles from './InventoryList.module.css';

// ステータスの日本語ラベルマッピング
const statusLabels: Record<InventoryStatus, string> = {
  in_stock: '在庫あり',
  low_stock: '低在庫',
  out_of_stock: '在庫切れ',
};

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<InventoryStatus, string> = {
  in_stock: 'statusInStock',
  low_stock: 'statusLowStock',
  out_of_stock: 'statusOutOfStock',
};

// 在庫一覧コンポーネント: テーブル表示でステータスフィルター機能を提供
export function InventoryList() {
  // ステータスフィルターの状態管理
  const [statusFilter, setStatusFilter] = useState<InventoryStatus | ''>('');

  const { data: items, isLoading, error } = useInventoryList();
  const navigate = useNavigate();

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // ステータスフィルターが適用された在庫アイテムリスト
  const filteredItems = statusFilter
    ? items?.filter((item) => item.status === statusFilter)
    : items;

  // 行クリック時に詳細画面へ遷移
  const handleRowClick = (id: string) => {
    navigate({ to: '/inventory/$id', params: { id } });
  };

  return (
    <main>
      <h1>在庫一覧</h1>

      {/* ステータスフィルターのツールバー */}
      <div className={styles.toolbar}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as InventoryStatus | '')}
          aria-label="ステータスでフィルター"
        >
          <option value="">すべて</option>
          <option value="in_stock">在庫あり</option>
          <option value="low_stock">低在庫</option>
          <option value="out_of_stock">在庫切れ</option>
        </select>
      </div>

      {/* 在庫一覧テーブル */}
      <table className={styles.table}>
        <thead>
          <tr>
            <th className={styles.th}>商品名</th>
            <th className={styles.th}>倉庫名</th>
            <th className={styles.th}>利用可能数</th>
            <th className={styles.th}>予約数</th>
            <th className={styles.th}>再注文点</th>
            <th className={styles.th}>ステータス</th>
          </tr>
        </thead>
        <tbody>
          {filteredItems?.map((item) => (
            <tr
              key={item.id}
              onClick={() => handleRowClick(item.id)}
              className={styles.clickableRow}
              role="button"
              tabIndex={0}
              aria-label={`${item.product_name}の詳細を表示`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') handleRowClick(item.id);
              }}
            >
              <td className={styles.td}>{item.product_name}</td>
              <td className={styles.td}>{item.warehouse_name}</td>
              <td className={styles.td}>{item.quantity_available}</td>
              <td className={styles.td}>{item.quantity_reserved}</td>
              <td className={styles.td}>{item.reorder_point}</td>
              <td className={styles.td}>
                <span className={`${styles.statusBadge} ${styles[statusClassMap[item.status]]}`}>
                  {statusLabels[item.status]}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {filteredItems?.length === 0 && <p>在庫データがありません。</p>}
    </main>
  );
}
