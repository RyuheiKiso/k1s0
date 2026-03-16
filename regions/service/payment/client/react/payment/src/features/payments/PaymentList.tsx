import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { usePayments } from '../../hooks/usePayments';
import type { PaymentStatus } from '../../types/payment';

// ステータスの日本語表示ラベルマッピング（サーバー契約に準拠: pending/processing→initiated）
const statusLabels: Record<PaymentStatus, string> = {
  initiated: '開始済',
  completed: '完了',
  failed: '失敗',
  refunded: '返金済',
};

// ステータスバッジの色マッピング（サーバー契約に準拠）
const statusColors: Record<PaymentStatus, { background: string; color: string }> = {
  initiated: { background: '#fff3cd', color: '#856404' },
  completed: { background: '#d4edda', color: '#155724' },
  failed: { background: '#f8d7da', color: '#721c24' },
  refunded: { background: '#e2e3e5', color: '#383d41' },
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
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  return (
    <div>
      <h1>決済一覧</h1>

      {/* ステータスフィルターのツールバー */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label htmlFor="status-filter">ステータス:</label>
        <select
          id="status-filter"
          value={statusFilter ?? ''}
          onChange={(e) =>
            setStatusFilter(e.target.value ? (e.target.value as PaymentStatus) : undefined)
          }
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
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={thStyle}>決済ID</th>
            <th style={thStyle}>注文ID</th>
            <th style={thStyle}>顧客ID</th>
            <th style={thStyle}>金額</th>
            <th style={thStyle}>ステータス</th>
            <th style={thStyle}>決済方法</th>
            <th style={thStyle}>作成日</th>
          </tr>
        </thead>
        <tbody>
          {payments?.map((payment) => (
            <tr
              key={payment.id}
              onClick={() => handleRowClick(payment.id)}
              style={{ cursor: 'pointer' }}
            >
              <td style={tdStyle}>{payment.id.substring(0, 8)}...</td>
              <td style={tdStyle}>{payment.order_id}</td>
              <td style={tdStyle}>{payment.customer_id}</td>
              <td style={tdStyle}>
                {payment.amount.toLocaleString()} {payment.currency}
              </td>
              <td style={tdStyle}>
                {/* ステータスバッジ: 色分けで視覚的に区別 */}
                <span
                  style={{
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontSize: '0.85em',
                    ...statusColors[payment.status],
                  }}
                >
                  {statusLabels[payment.status]}
                </span>
              </td>
              {/* 決済方法表示: ラベルマップにフォールバック付き */}
              <td style={tdStyle}>
                {payment.payment_method
                  ? (paymentMethodLabels[payment.payment_method] ?? payment.payment_method)
                  : '-'}
              </td>
              <td style={tdStyle}>{new Date(payment.created_at).toLocaleString('ja-JP')}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* データが空の場合のメッセージ */}
      {payments?.length === 0 && <p>決済データがありません。</p>}
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
