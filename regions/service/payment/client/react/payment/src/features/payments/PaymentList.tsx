import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { usePayments } from '../../hooks/usePayments';
import type { PaymentStatus } from '../../types/payment';
import styles from './PaymentList.module.css';

// ステータスの日本語表示ラベルマッピング（サーバー契約に準拠: pending/processing→initiated）
const statusLabels: Record<PaymentStatus, string> = {
  initiated: '開始済',
  completed: '完了',
  failed: '失敗',
  refunded: '返金済',
};

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<PaymentStatus, string> = {
  initiated: 'statusInitiated',
  completed: 'statusCompleted',
  failed: 'statusFailed',
  refunded: 'statusRefunded',
};

// 決済方法の日本語表示ラベルマッピング
const paymentMethodLabels: Record<string, string> = {
  credit_card: 'クレジットカード',
  bank_transfer: '銀行振込',
  convenience_store: 'コンビニ払い',
  e_wallet: '電子ウォレット',
};

// 決済一覧コンポーネント: テーブル表示でフィルタリング機能を提供
export function PaymentList() {
  // ステータスフィルターの状態管理
  const [statusFilter, setStatusFilter] = useState<PaymentStatus | undefined>(undefined);
  const navigate = useNavigate();

  const { data: payments, isLoading, error } = usePayments(undefined, undefined, statusFilter);

  // 決済行クリック時の詳細画面遷移
  const handleRowClick = (id: string) => {
    navigate({ to: '/payments/$id', params: { id } });
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  return (
    <main>
      <h1>決済一覧</h1>

      {/* ステータスフィルターのツールバー */}
      <div className={styles.toolbar}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as PaymentStatus) : undefined)
          }
          aria-label="ステータスでフィルター"
        >
          <option value="">すべて</option>
          {/* サーバー契約に準拠したステータス選択肢 */}
          <option value="initiated">開始済</option>
          <option value="completed">完了</option>
          <option value="failed">失敗</option>
          <option value="refunded">返金済</option>
        </select>
      </div>

      {/* 決済一覧テーブル */}
      <table className={styles.table} aria-label="決済一覧">
        <thead>
          <tr>
            <th className={styles.th}>決済ID</th>
            <th className={styles.th}>注文ID</th>
            <th className={styles.th}>顧客ID</th>
            <th className={styles.th}>金額</th>
            <th className={styles.th}>ステータス</th>
            <th className={styles.th}>決済方法</th>
            <th className={styles.th}>作成日</th>
          </tr>
        </thead>
        <tbody>
          {payments?.map((payment) => (
            <tr
              key={payment.id}
              onClick={() => handleRowClick(payment.id)}
              className={styles.clickableRow}
              role="button"
              tabIndex={0}
              aria-label={`決済 ${payment.id.substring(0, 8)} の詳細を表示`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') handleRowClick(payment.id);
              }}
            >
              <td className={styles.td}>{payment.id.substring(0, 8)}...</td>
              <td className={styles.td}>{payment.order_id}</td>
              <td className={styles.td}>{payment.customer_id}</td>
              <td className={styles.td}>
                {payment.amount.toLocaleString()} {payment.currency}
              </td>
              <td className={styles.td}>
                {/* ステータスバッジ: 色分けで視覚的に区別 */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[payment.status]]}`}>
                  {statusLabels[payment.status]}
                </span>
              </td>
              {/* 決済方法表示: ラベルマップにフォールバック付き */}
              <td className={styles.td}>
                {payment.payment_method
                  ? (paymentMethodLabels[payment.payment_method] ?? payment.payment_method)
                  : '-'}
              </td>
              <td className={styles.td}>{new Date(payment.created_at).toLocaleString('ja-JP')}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {payments?.length === 0 && <p>決済データがありません。</p>}
    </main>
  );
}
