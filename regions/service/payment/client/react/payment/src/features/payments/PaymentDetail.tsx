import { useState } from 'react';
import { usePayment, useCompletePayment, useFailPayment, useRefundPayment } from '../../hooks/usePayments';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import { InputDialog } from '../../components/InputDialog';
import type { PaymentStatus } from '../../types/payment';
import styles from './PaymentDetail.module.css';

// 決済詳細コンポーネントのProps
interface PaymentDetailProps {
  id: string;
}

// ステータスの日本語表示ラベルマッピング（サーバー契約に準拠: pending/processing→initiated）
const statusLabels: Record<PaymentStatus, string> = {
  initiated: '開始済',
  completed: '完了',
  failed: '失敗',
  refunded: '返金済',
};

// ステータスバッジのCSSクラス名マッピング（サーバー契約に準拠）
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

// 決済完了ダイアログの入力フィールド定義
const completeFields = [
  { key: 'transaction_id', label: 'トランザクションID', required: true, placeholder: 'トランザクションIDを入力' },
];

// 決済失敗ダイアログの入力フィールド定義
const failFields = [
  { key: 'error_code', label: 'エラーコード', required: true, placeholder: 'エラーコードを入力' },
  { key: 'error_message', label: 'エラーメッセージ', required: false, placeholder: 'エラーメッセージを入力' },
];

// 返金ダイアログの入力フィールド定義
const refundFields = [
  { key: 'reason', label: '返金理由（任意）', required: false, placeholder: '返金理由を入力' },
];

// 決済詳細コンポーネント: 決済情報の表示とステータス変更アクションを提供
export function PaymentDetail({ id }: PaymentDetailProps) {
  const { data: payment, isLoading, error } = usePayment(id);
  const completePayment = useCompletePayment();
  const failPayment = useFailPayment();
  const refundPayment = useRefundPayment();

  // ダイアログの表示状態管理
  const [completeDialogOpen, setCompleteDialogOpen] = useState(false);
  const [failDialogOpen, setFailDialogOpen] = useState(false);
  const [refundInputOpen, setRefundInputOpen] = useState(false);
  const [refundConfirmOpen, setRefundConfirmOpen] = useState(false);
  // 返金理由の一時保持（入力ダイアログ→確認ダイアログの間で保持）
  const [pendingRefundReason, setPendingRefundReason] = useState<string | undefined>(undefined);

  // 決済完了: 入力ダイアログでtransaction_idを受け取り、APIを呼び出す
  const handleCompleteSubmit = (values: Record<string, string>) => {
    setCompleteDialogOpen(false);
    if (values.transaction_id) {
      completePayment.mutate({ id, transaction_id: values.transaction_id });
    }
  };

  // 決済失敗: 入力ダイアログでerror_code/error_messageを受け取り、APIを呼び出す
  const handleFailSubmit = (values: Record<string, string>) => {
    setFailDialogOpen(false);
    if (values.error_code) {
      failPayment.mutate({ id, error_code: values.error_code, error_message: values.error_message ?? '' });
    }
  };

  // 返金: 入力ダイアログでreasonを受け取り、確認ダイアログへ遷移
  const handleRefundInputSubmit = (values: Record<string, string>) => {
    setRefundInputOpen(false);
    setPendingRefundReason(values.reason || undefined);
    setRefundConfirmOpen(true);
  };

  // 返金確認: 確認ダイアログでOKならAPIを呼び出す
  const handleRefundConfirm = () => {
    setRefundConfirmOpen(false);
    refundPayment.mutate({ id, reason: pendingRefundReason });
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // データが見つからない場合の表示
  if (!payment) return <div>決済データが見つかりません。</div>;

  // ステータスに応じてアクションボタンの表示を制御（initiated時のみ完了/失敗可能）
  const canComplete = payment.status === 'initiated';
  const canFail = payment.status === 'initiated';
  const canRefund = payment.status === 'completed';

  return (
    <main>
      <h1>決済詳細</h1>

      {/* 決済情報テーブル */}
      <table className={styles.infoTable}>
        <tbody>
          <tr>
            <th className={styles.labelCell}>決済ID</th>
            <td className={styles.valueCell}>{payment.id}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>注文ID</th>
            <td className={styles.valueCell}>{payment.order_id}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>顧客ID</th>
            <td className={styles.valueCell}>{payment.customer_id}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>金額</th>
            <td className={styles.valueCell}>
              {payment.amount.toLocaleString()} {payment.currency}
            </td>
          </tr>
          <tr>
            <th className={styles.labelCell}>ステータス</th>
            <td className={styles.valueCell}>
              {/* ステータスバッジ */}
              <span className={`${styles.statusBadge} ${styles[statusClassMap[payment.status]]}`}>
                {statusLabels[payment.status]}
              </span>
            </td>
          </tr>
          <tr>
            <th className={styles.labelCell}>決済方法</th>
            {/* 決済方法表示: ラベルマップにフォールバック付き */}
            <td className={styles.valueCell}>
              {payment.payment_method
                ? (paymentMethodLabels[payment.payment_method] ?? payment.payment_method)
                : '-'}
            </td>
          </tr>
          <tr>
            <th className={styles.labelCell}>トランザクションID</th>
            <td className={styles.valueCell}>{payment.transaction_id ?? '-'}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>エラーコード</th>
            <td className={styles.valueCell}>{payment.error_code ?? '-'}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>エラーメッセージ</th>
            <td className={styles.valueCell}>{payment.error_message ?? '-'}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>バージョン</th>
            <td className={styles.valueCell}>{payment.version}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>作成日</th>
            <td className={styles.valueCell}>{new Date(payment.created_at).toLocaleString('ja-JP')}</td>
          </tr>
          <tr>
            <th className={styles.labelCell}>更新日</th>
            <td className={styles.valueCell}>{new Date(payment.updated_at).toLocaleString('ja-JP')}</td>
          </tr>
        </tbody>
      </table>

      {/* ステータス変更アクションボタン */}
      <div className={styles.actions}>
        {/* 完了ボタン: initiated時のみ表示 */}
        {canComplete && (
          <button
            onClick={() => setCompleteDialogOpen(true)}
            disabled={completePayment.isPending}
            className={styles.completeButton}
            aria-label="決済を完了する"
          >
            完了
          </button>
        )}

        {/* 失敗ボタン: initiated時のみ表示 */}
        {canFail && (
          <button
            onClick={() => setFailDialogOpen(true)}
            disabled={failPayment.isPending}
            className={styles.failButton}
            aria-label="決済を失敗にする"
          >
            失敗
          </button>
        )}

        {/* 返金ボタン: completed時のみ表示 */}
        {canRefund && (
          <button
            onClick={() => setRefundInputOpen(true)}
            disabled={refundPayment.isPending}
            className={styles.refundButton}
            aria-label="決済を返金する"
          >
            返金
          </button>
        )}
      </div>

      {/* ミューテーションエラー表示 */}
      {(completePayment.error || failPayment.error || refundPayment.error) && (
        <p className={styles.errorMessage} role="alert">
          操作に失敗しました:{' '}
          {((completePayment.error || failPayment.error || refundPayment.error) as Error).message}
        </p>
      )}

      {/* 決済完了入力ダイアログ */}
      <InputDialog
        open={completeDialogOpen}
        title="決済完了"
        fields={completeFields}
        submitLabel="完了する"
        onSubmit={handleCompleteSubmit}
        onCancel={() => setCompleteDialogOpen(false)}
      />

      {/* 決済失敗入力ダイアログ */}
      <InputDialog
        open={failDialogOpen}
        title="決済失敗"
        fields={failFields}
        submitLabel="失敗にする"
        onSubmit={handleFailSubmit}
        onCancel={() => setFailDialogOpen(false)}
      />

      {/* 返金理由入力ダイアログ */}
      <InputDialog
        open={refundInputOpen}
        title="返金"
        fields={refundFields}
        submitLabel="次へ"
        onSubmit={handleRefundInputSubmit}
        onCancel={() => setRefundInputOpen(false)}
      />

      {/* 返金確認ダイアログ */}
      <ConfirmDialog
        open={refundConfirmOpen}
        title="返金確認"
        message="この決済を返金しますか？この操作は取り消せません。"
        confirmLabel="返金する"
        onConfirm={handleRefundConfirm}
        onCancel={() => setRefundConfirmOpen(false)}
      />
    </main>
  );
}
