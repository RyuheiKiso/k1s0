import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useOrders } from '../../hooks/useOrders';
import type { OrderStatus } from '../../types/order';

// ステータス表示ラベルのマッピング
const statusLabels: Record<OrderStatus, string> = {
  pending: '保留中',
  confirmed: '確認済',
  processing: '処理中',
  shipped: '発送済',
  delivered: '配達済',
  cancelled: 'キャンセル',
};

// ステータスバッジの背景色マッピング
const statusColors: Record<OrderStatus, string> = {
  pending: '#ffc107',
  confirmed: '#17a2b8',
  processing: '#007bff',
  shipped: '#6f42c1',
  delivered: '#28a745',
  cancelled: '#dc3545',
};

// 注文一覧コンポーネント: テーブル表示でステータスフィルタ機能を提供
export function OrderList() {
  // ステータスフィルターの状態管理
  const [statusFilter, setStatusFilter] = useState<OrderStatus | undefined>(undefined);
  const navigate = useNavigate();

  const { data: orders, isLoading, error } = useOrders(undefined, statusFilter);

  // 注文行クリック時に詳細画面へ遷移
  const handleRowClick = (id: string) => {
    navigate({ to: '/orders/$id', params: { id } });
  };

  // 金額をフォーマットして表示
  const formatAmount = (amount: number, currency: string) => {
    return new Intl.NumberFormat('ja-JP', {
      style: 'currency',
      currency,
    }).format(amount);
  };

  // 日付をフォーマットして表示
  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      <h1>注文一覧</h1>

      {/* ステータスフィルタードロップダウン */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as OrderStatus) : undefined)
          }
        >
          <option value="">すべて</option>
          <option value="pending">保留中</option>
          <option value="confirmed">確認済</option>
          <option value="processing">処理中</option>
          <option value="shipped">発送済</option>
          <option value="delivered">配達済</option>
          <option value="cancelled">キャンセル</option>
        </select>
      </div>

      {/* 注文一覧テーブル */}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>注文ID</th>
            <th style={thStyle}>顧客ID</th>
            <th style={thStyle}>ステータス</th>
            <th style={thStyle}>合計金額</th>
            <th style={thStyle}>作成日</th>
          </tr>
        </thead>
        <tbody>
          {orders?.map((order) => (
            <tr
              key={order.id}
              onClick={() => handleRowClick(order.id)}
              style={{ cursor: 'pointer' }}
            >
              <td style={tdStyle}>{order.id.substring(0, 8)}...</td>
              <td style={tdStyle}>{order.customer_id}</td>
              <td style={tdStyle}>
                {/* ステータスバッジ: ステータスに応じた色で表示 */}
                <span
                  style={{
                    backgroundColor: statusColors[order.status],
                    color: '#fff',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontSize: '0.85em',
                  }}
                >
                  {statusLabels[order.status]}
                </span>
              </td>
              <td style={tdStyle}>{formatAmount(order.total_amount, order.currency)}</td>
              <td style={tdStyle}>{formatDate(order.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {orders?.length === 0 && <p>注文がありません。</p>}
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
