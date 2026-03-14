import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useInventoryList } from '../../hooks/useInventory';
import type { InventoryStatus } from '../../types/inventory';

// ステータスの日本語ラベルマッピング
const statusLabels: Record<InventoryStatus, string> = {
  in_stock: '在庫あり',
  low_stock: '低在庫',
  out_of_stock: '在庫切れ',
};

// ステータスバッジの色マッピング
const statusColors: Record<InventoryStatus, React.CSSProperties> = {
  in_stock: { backgroundColor: '#d4edda', color: '#155724', padding: '2px 8px', borderRadius: '4px' },
  low_stock: { backgroundColor: '#fff3cd', color: '#856404', padding: '2px 8px', borderRadius: '4px' },
  out_of_stock: { backgroundColor: '#f8d7da', color: '#721c24', padding: '2px 8px', borderRadius: '4px' },
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
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  // ステータスフィルターが適用された在庫アイテムリスト
  const filteredItems = statusFilter
    ? items?.filter((item) => item.status === statusFilter)
    : items;

  // 行クリック時に詳細画面へ遷移
  const handleRowClick = (id: string) => {
    navigate({ to: '/inventory/$id', params: { id } });
  };

  return (
    <div>
      <h1>在庫一覧</h1>

      {/* ステータスフィルターのツールバー */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as InventoryStatus | '')}
        >
          <option value="">すべて</option>
          <option value="in_stock">在庫あり</option>
          <option value="low_stock">低在庫</option>
          <option value="out_of_stock">在庫切れ</option>
        </select>
      </div>

      {/* 在庫一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>商品名</th>
            <th style={thStyle}>倉庫名</th>
            <th style={thStyle}>利用可能数</th>
            <th style={thStyle}>予約数</th>
            <th style={thStyle}>再注文点</th>
            <th style={thStyle}>ステータス</th>
          </tr>
        </thead>
        <tbody>
          {filteredItems?.map((item) => (
            <tr
              key={item.id}
              onClick={() => handleRowClick(item.id)}
              style={{ cursor: 'pointer' }}
            >
              <td style={tdStyle}>{item.product_name}</td>
              <td style={tdStyle}>{item.warehouse_name}</td>
              <td style={tdStyle}>{item.quantity_available}</td>
              <td style={tdStyle}>{item.quantity_reserved}</td>
              <td style={tdStyle}>{item.reorder_point}</td>
              <td style={tdStyle}>
                <span style={statusColors[item.status]}>
                  {statusLabels[item.status]}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {filteredItems?.length === 0 && <p>在庫データがありません。</p>}
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
