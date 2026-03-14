import { usePayment, useCompletePayment, useFailPayment, useRefundPayment } from '../../hooks/usePayments';
import type { PaymentStatus } from '../../types/payment';

// 決済詳細コンポーネントのProps
interface PaymentDetailProps {
  id: string;
}

// ステータスの日本語表示ラベルマッピング
const statusLabels: Record<PaymentStatus, string> = {
  pending: '保留中',
  processing: '処理中',
  completed: '完了',
  failed: '失敗',
  refunded: '返金済',
};

// ステータスバッジの色マッピング
const statusColors: Record<PaymentStatus, { background: string; color: string }> = {
  pending: { background: '#fff3cd', color: '#856404' },
  processing: { background: '#cce5ff', color: '#004085' },
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

// 決済詳細コンポーネント: 決済情報の表示とステータス変更アクションを提供
export function PaymentDetail({ id }: PaymentDetailProps) {
  const { data: payment, isLoading, error } = usePayment(id);
  const completePayment = useCompletePayment();
  const failPayment = useFailPayment();
  const refundPayment = useRefundPayment();

  // 決済完了の確認ダイアログと実行
  const handleComplete = () => {
    if (window.confirm('この決済を完了しますか？')) {
      completePayment.mutate(id);
    }
  };

  // 決済失敗の確認ダイアログと実行
  const handleFail = () => {
    if (window.confirm('この決済を失敗にしますか？')) {
      failPayment.mutate(id);
    }
  };

  // 決済返金の確認ダイアログと実行
  const handleRefund = () => {
    if (window.confirm('この決済を返金しますか？この操作は取り消せません。')) {
      refundPayment.mutate(id);
    }
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  // データが見つからない場合の表示
  if (!payment) return <div>決済データが見つかりません。</div>;

  // ステータスに応じてアクションボタンの表示を制御
  const canComplete = payment.status === 'pending' || payment.status === 'processing';
  const canFail = payment.status === 'pending' || payment.status === 'processing';
  const canRefund = payment.status === 'completed';

  return (
    <div>
      <h1>決済詳細</h1>

      {/* 決済情報テーブル */}
      <table style={{ borderCollapse: 'collapse', marginBottom: '24px' }}>
        <tbody>
          <tr>
            <th style={labelStyle}>決済ID</th>
            <td style={valueStyle}>{payment.id}</td>
          </tr>
          <tr>
            <th style={labelStyle}>注文ID</th>
            <td style={valueStyle}>{payment.order_id}</td>
          </tr>
          <tr>
            <th style={labelStyle}>顧客ID</th>
            <td style={valueStyle}>{payment.customer_id}</td>
          </tr>
          <tr>
            <th style={labelStyle}>金額</th>
            <td style={valueStyle}>
              {payment.amount.toLocaleString()} {payment.currency}
            </td>
          </tr>
          <tr>
            <th style={labelStyle}>ステータス</th>
            <td style={valueStyle}>
              {/* ステータスバッジ */}
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
          </tr>
          <tr>
            <th style={labelStyle}>決済方法</th>
            <td style={valueStyle}>{paymentMethodLabels[payment.payment_method]}</td>
          </tr>
          <tr>
            <th style={labelStyle}>トランザクションID</th>
            <td style={valueStyle}>{payment.transaction_id ?? '-'}</td>
          </tr>
          <tr>
            <th style={labelStyle}>失敗理由</th>
            <td style={valueStyle}>{payment.failure_reason ?? '-'}</td>
          </tr>
          <tr>
            <th style={labelStyle}>返金額</th>
            <td style={valueStyle}>
              {payment.refund_amount !== null
                ? `${payment.refund_amount.toLocaleString()} ${payment.currency}`
                : '-'}
            </td>
          </tr>
          <tr>
            <th style={labelStyle}>作成日</th>
            <td style={valueStyle}>{new Date(payment.created_at).toLocaleString('ja-JP')}</td>
          </tr>
          <tr>
            <th style={labelStyle}>更新日</th>
            <td style={valueStyle}>{new Date(payment.updated_at).toLocaleString('ja-JP')}</td>
          </tr>
        </tbody>
      </table>

      {/* ステータス変更アクションボタン */}
      <div style={{ display: 'flex', gap: '8px' }}>
        {/* 完了ボタン: pending/processing時のみ表示 */}
        {canComplete && (
          <button
            onClick={handleComplete}
            disabled={completePayment.isPending}
            style={{ backgroundColor: '#28a745', color: 'white', border: 'none', padding: '8px 16px', borderRadius: '4px', cursor: 'pointer' }}
          >
            完了
          </button>
        )}

        {/* 失敗ボタン: pending/processing時のみ表示 */}
        {canFail && (
          <button
            onClick={handleFail}
            disabled={failPayment.isPending}
            style={{ backgroundColor: '#dc3545', color: 'white', border: 'none', padding: '8px 16px', borderRadius: '4px', cursor: 'pointer' }}
          >
            失敗
          </button>
        )}

        {/* 返金ボタン: completed時のみ表示 */}
        {canRefund && (
          <button
            onClick={handleRefund}
            disabled={refundPayment.isPending}
            style={{ backgroundColor: '#6c757d', color: 'white', border: 'none', padding: '8px 16px', borderRadius: '4px', cursor: 'pointer' }}
          >
            返金
          </button>
        )}
      </div>

      {/* ミューテーションエラー表示 */}
      {(completePayment.error || failPayment.error || refundPayment.error) && (
        <p style={{ color: 'red', marginTop: '8px' }}>
          操作に失敗しました:{' '}
          {((completePayment.error || failPayment.error || refundPayment.error) as Error).message}
        </p>
      )}
    </div>
  );
}

// ラベルセルのスタイル
const labelStyle: React.CSSProperties = {
  padding: '8px 16px',
  textAlign: 'left',
  borderBottom: '1px solid #eee',
  backgroundColor: '#f8f9fa',
  whiteSpace: 'nowrap',
};

// 値セルのスタイル
const valueStyle: React.CSSProperties = {
  padding: '8px 16px',
  borderBottom: '1px solid #eee',
};
