import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useOrders } from '../../hooks/useOrders';
import type { OrderStatus } from '../../types/order';
import styles from './OrderList.module.css';

// ステータス表示ラベルのマッピング
const statusLabels: Record<OrderStatus, string> = {
  pending: '保留中',
  confirmed: '確認済',
  processing: '処理中',
  shipped: '発送済',
  delivered: '配達済',
  cancelled: 'キャンセル',
};

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<OrderStatus, string> = {
  pending: 'statusPending',
  confirmed: 'statusConfirmed',
  processing: 'statusProcessing',
  shipped: 'statusShipped',
  delivered: 'statusDelivered',
  cancelled: 'statusCancelled',
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
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  return (
    <main>
      <h1>注文一覧</h1>

      {/* ステータスフィルタードロップダウン */}
      <div className={styles.toolbar}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as OrderStatus) : undefined)
          }
          aria-label="ステータスでフィルター"
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
      <table className={styles.table} aria-label="注文一覧">
        <thead>
          <tr>
            <th className={styles.th}>注文ID</th>
            <th className={styles.th}>顧客ID</th>
            <th className={styles.th}>ステータス</th>
            <th className={styles.th}>合計金額</th>
            <th className={styles.th}>作成日</th>
          </tr>
        </thead>
        <tbody>
          {orders?.map((order) => (
            <tr
              key={order.id}
              onClick={() => handleRowClick(order.id)}
              className={styles.clickableRow}
              role="button"
              tabIndex={0}
              aria-label={`注文 ${order.id.substring(0, 8)} の詳細を表示`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') handleRowClick(order.id);
              }}
            >
              <td className={styles.td}>{order.id.substring(0, 8)}...</td>
              <td className={styles.td}>{order.customer_id}</td>
              <td className={styles.td}>
                {/* ステータスバッジ: ステータスに応じた色で表示 */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[order.status]]}`}>
                  {statusLabels[order.status]}
                </span>
              </td>
              <td className={styles.td}>{formatAmount(order.total_amount, order.currency)}</td>
              <td className={styles.td}>{formatDate(order.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {orders?.length === 0 && <p>注文がありません。</p>}
    </main>
  );
}
