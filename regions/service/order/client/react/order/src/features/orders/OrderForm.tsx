import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { createOrderInputSchema } from '../../types/order';
import { useCreateOrder } from '../../hooks/useOrders';
import styles from './OrderForm.module.css';

// 注文アイテム行の入力状態の型
interface ItemRow {
  product_id: string;
  product_name: string;
  quantity: number;
  unit_price: number;
}

// 空の注文アイテム行を生成
function emptyItemRow(): ItemRow {
  return { product_id: '', product_name: '', quantity: 1, unit_price: 0 };
}

// 注文作成フォームコンポーネント: 動的なアイテムリストとZodバリデーション付き
export function OrderForm() {
  const navigate = useNavigate();

  // フォーム入力値の状態管理
  const [customerId, setCustomerId] = useState('');
  const [currency, setCurrency] = useState('JPY');
  const [notes, setNotes] = useState('');
  const [items, setItems] = useState<ItemRow[]>([emptyItemRow()]);
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createOrder = useCreateOrder();

  // アイテム行のフィールド値を更新
  const updateItem = (index: number, field: keyof ItemRow, value: string | number) => {
    setItems((prev) => {
      const updated = [...prev];
      updated[index] = { ...updated[index], [field]: value };
      return updated;
    });
  };

  // アイテム行を追加
  const addItem = () => {
    setItems((prev) => [...prev, emptyItemRow()]);
  };

  // アイテム行を削除（最低1行は残す）
  const removeItem = (index: number) => {
    if (items.length <= 1) return;
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  // アイテムの小計を計算
  const calcSubtotal = (item: ItemRow) => item.quantity * item.unit_price;

  // 全アイテムの合計金額を計算
  const calcTotal = () => items.reduce((sum, item) => sum + calcSubtotal(item), 0);

  // 金額をフォーマットして表示
  const formatAmount = (amount: number) => {
    return new Intl.NumberFormat('ja-JP', {
      style: 'currency',
      currency,
    }).format(amount);
  };

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    const input = {
      customer_id: customerId,
      currency: currency || undefined,
      items: items.map((item) => ({
        product_id: item.product_id,
        product_name: item.product_name,
        quantity: item.quantity,
        unit_price: item.unit_price,
      })),
      notes: notes || undefined,
    };

    // Zodスキーマでバリデーション実行
    const result = createOrderInputSchema.safeParse(input);

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

    // API呼び出し: 成功時に注文詳細画面へ遷移
    createOrder.mutate(result.data, {
      onSuccess: (order) => {
        navigate({ to: '/orders/$id', params: { id: order.id } });
      },
    });
  };

  return (
    <main>
      <h1>新規注文作成</h1>
      <form onSubmit={handleSubmit}>
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

        {/* 通貨選択 */}
        <div className={styles.field}>
          <label htmlFor="currency">通貨</label>
          <select id="currency" value={currency} onChange={(e) => setCurrency(e.target.value)} aria-label="通貨を選択">
            <option value="JPY">JPY (日本円)</option>
            <option value="USD">USD (米ドル)</option>
            <option value="EUR">EUR (ユーロ)</option>
          </select>
        </div>

        {/* 注文アイテムリスト */}
        <section className={styles.itemsSection} aria-label="注文アイテム">
          <h2>注文アイテム</h2>
          {errors.items && <span className={styles.error} role="alert">{errors.items}</span>}

          <table className={styles.table}>
            <thead>
              <tr>
                <th className={styles.th}>商品ID</th>
                <th className={styles.th}>商品名</th>
                <th className={styles.th}>数量</th>
                <th className={styles.th}>単価</th>
                <th className={styles.th}>小計</th>
                <th className={styles.th}>操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item, index) => (
                <tr key={index}>
                  {/* 商品ID入力欄 */}
                  <td className={styles.td}>
                    <input
                      value={item.product_id}
                      onChange={(e) => updateItem(index, 'product_id', e.target.value)}
                      placeholder="商品ID"
                      className={styles.fullWidthInput}
                      aria-label={`商品ID (アイテム ${index + 1})`}
                    />
                    {errors[`items.${index}.product_id`] && (
                      <span className={styles.error} role="alert">{errors[`items.${index}.product_id`]}</span>
                    )}
                  </td>
                  {/* 商品名入力欄 */}
                  <td className={styles.td}>
                    <input
                      value={item.product_name}
                      onChange={(e) => updateItem(index, 'product_name', e.target.value)}
                      placeholder="商品名"
                      className={styles.fullWidthInput}
                      aria-label={`商品名 (アイテム ${index + 1})`}
                    />
                    {errors[`items.${index}.product_name`] && (
                      <span className={styles.error} role="alert">{errors[`items.${index}.product_name`]}</span>
                    )}
                  </td>
                  {/* 数量入力欄 */}
                  <td className={styles.td}>
                    <input
                      type="number"
                      value={item.quantity}
                      onChange={(e) => updateItem(index, 'quantity', Number(e.target.value))}
                      min={1}
                      className={styles.quantityInput}
                      aria-label={`数量 (アイテム ${index + 1})`}
                    />
                    {errors[`items.${index}.quantity`] && (
                      <span className={styles.error} role="alert">{errors[`items.${index}.quantity`]}</span>
                    )}
                  </td>
                  {/* 単価入力欄 */}
                  <td className={styles.td}>
                    <input
                      type="number"
                      value={item.unit_price}
                      onChange={(e) => updateItem(index, 'unit_price', Number(e.target.value))}
                      min={0}
                      className={styles.priceInput}
                      aria-label={`単価 (アイテム ${index + 1})`}
                    />
                    {errors[`items.${index}.unit_price`] && (
                      <span className={styles.error} role="alert">{errors[`items.${index}.unit_price`]}</span>
                    )}
                  </td>
                  {/* 小計（自動計算） */}
                  <td className={styles.td}>{formatAmount(calcSubtotal(item))}</td>
                  {/* 削除ボタン */}
                  <td className={styles.td}>
                    <button
                      type="button"
                      onClick={() => removeItem(index)}
                      disabled={items.length <= 1}
                      className={styles.removeButton}
                      aria-label={`アイテム ${index + 1} を削除`}
                    >
                      削除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {/* アイテム追加ボタン */}
          <button type="button" onClick={addItem} aria-label="アイテムを追加">
            アイテムを追加
          </button>
        </section>

        {/* 合計金額表示 */}
        <div className={styles.total}>
          合計: {formatAmount(calcTotal())}
        </div>

        {/* 備考入力欄 */}
        <div className={styles.field}>
          <label htmlFor="notes">備考</label>
          <textarea
            id="notes"
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            rows={3}
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div className={styles.actions}>
          <button type="submit" disabled={createOrder.isPending} aria-label="注文を作成">
            注文を作成
          </button>
          <button type="button" onClick={() => navigate({ to: '/' })} aria-label="キャンセル">
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {createOrder.error && (
          <p className={styles.error} role="alert">
            注文の作成に失敗しました: {(createOrder.error as Error).message}
          </p>
        )}
      </form>
    </main>
  );
}
