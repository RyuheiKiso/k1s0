import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { initiatePaymentSchema } from '../../types/payment';
import { useInitiatePayment } from '../../hooks/usePayments';
import styles from './PaymentForm.module.css';

// 決済方法の選択肢定義（サーバー契約に準拠: enumではなくstring値）
const paymentMethodOptions: { value: string; label: string }[] = [
  { value: 'credit_card', label: 'クレジットカード' },
  { value: 'bank_transfer', label: '銀行振込' },
  { value: 'convenience_store', label: 'コンビニ払い' },
  { value: 'e_wallet', label: '電子ウォレット' },
];

// 決済開始フォームコンポーネント: Zodバリデーション付き
export function PaymentForm() {
  const navigate = useNavigate();

  // フォーム入力値の状態管理
  const [orderId, setOrderId] = useState('');
  const [customerId, setCustomerId] = useState('');
  const [amount, setAmount] = useState<number>(0);
  const [currency, setCurrency] = useState('JPY');
  // 決済方法: サーバー契約に合わせてstring型に変更
  const [paymentMethod, setPaymentMethod] = useState<string>('credit_card');
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const initiatePayment = useInitiatePayment();

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      order_id: orderId,
      customer_id: customerId,
      amount,
      currency: currency || undefined,
      payment_method: paymentMethod || undefined,
    };

    // Zodスキーマでバリデーション実行
    const result = initiatePaymentSchema.safeParse(input);

    if (!result.success) {
      // バリデーションエラーをフィールド別に整理
      const fieldErrors: Record<string, string> = {};
      result.error.errors.forEach((err) => {
        const field = err.path.join('.');
        fieldErrors[field] = err.message;
      });
      setErrors(fieldErrors);
      return;
    }

    // API呼び出し: 成功時に決済詳細画面へ遷移
    initiatePayment.mutate(result.data, {
      onSuccess: (payment) => {
        navigate({ to: '/payments/$id', params: { id: payment.id } });
      },
    });
  };

  return (
    <main>
      <h1>新規決済</h1>
      <div className={styles.formContainer}>
        <form onSubmit={handleSubmit}>
          {/* 注文ID入力欄 */}
          <div className={styles.field}>
            <label htmlFor="order_id">注文ID</label>
            <input
              id="order_id"
              value={orderId}
              onChange={(e) => setOrderId(e.target.value)}
              required
              aria-required="true"
            />
            {errors.order_id && <span className={styles.error} role="alert">{errors.order_id}</span>}
          </div>

          {/* 顧客ID入力欄 */}
          <div className={styles.field}>
            <label htmlFor="customer_id">顧客ID</label>
            <input
              id="customer_id"
              value={customerId}
              onChange={(e) => setCustomerId(e.target.value)}
              required
              aria-required="true"
            />
            {errors.customer_id && <span className={styles.error} role="alert">{errors.customer_id}</span>}
          </div>

          {/* 金額入力欄 */}
          <div className={styles.field}>
            <label htmlFor="amount">金額</label>
            <input
              id="amount"
              type="number"
              value={amount}
              onChange={(e) => setAmount(Number(e.target.value))}
              min={1}
              required
              aria-required="true"
            />
            {errors.amount && <span className={styles.error} role="alert">{errors.amount}</span>}
          </div>

          {/* 通貨選択欄 */}
          <div className={styles.field}>
            <label htmlFor="currency">通貨</label>
            <input
              id="currency"
              value={currency}
              onChange={(e) => setCurrency(e.target.value)}
              aria-label="通貨コードを入力"
            />
          </div>

          {/* 決済方法選択欄 */}
          <div className={styles.field}>
            <label htmlFor="payment_method">決済方法</label>
            <select
              id="payment_method"
              value={paymentMethod}
              onChange={(e) => setPaymentMethod(e.target.value)}
              aria-label="決済方法を選択"
            >
              {paymentMethodOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            {errors.payment_method && <span className={styles.error} role="alert">{errors.payment_method}</span>}
          </div>

          {/* 送信・キャンセルボタン */}
          <div className={styles.actions}>
            <button type="submit" disabled={initiatePayment.isPending} aria-label="決済を開始">
              決済を開始
            </button>
            <button type="button" onClick={() => navigate({ to: '/' })} aria-label="キャンセル">
              キャンセル
            </button>
          </div>

          {/* APIエラー表示 */}
          {initiatePayment.error && (
            <p className={styles.error} role="alert">
              決済の開始に失敗しました: {(initiatePayment.error as Error).message}
            </p>
          )}
        </form>
      </div>
    </main>
  );
}
