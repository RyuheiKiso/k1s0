import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { createOrderInputSchema } from '../../types/order';
import { useCreateOrder } from '../../hooks/useOrders';

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
    <div>
      <h1>新規注文作成</h1>
      <form onSubmit={handleSubmit}>
        {/* 顧客ID入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="customer_id">顧客ID</label>
          <input
            id="customer_id"
            value={customerId}
            onChange={(e) => setCustomerId(e.target.value)}
            required
          />
          {errors.customer_id && <span style={errorStyle}>{errors.customer_id}</span>}
        </div>

        {/* 通貨選択 */}
        <div style={fieldStyle}>
          <label htmlFor="currency">通貨</label>
          <select id="currency" value={currency} onChange={(e) => setCurrency(e.target.value)}>
            <option value="JPY">JPY (日本円)</option>
            <option value="USD">USD (米ドル)</option>
            <option value="EUR">EUR (ユーロ)</option>
          </select>
        </div>

        {/* 注文アイテムリスト */}
        <div style={{ marginBottom: '16px' }}>
          <h2>注文アイテム</h2>
          {errors.items && <span style={errorStyle}>{errors.items}</span>}

          <table style={{ width: '100%', borderCollapse: 'collapse', marginBottom: '8px' }}>
            <thead>
              <tr>
                <th style={thStyle}>商品ID</th>
                <th style={thStyle}>商品名</th>
                <th style={thStyle}>数量</th>
                <th style={thStyle}>単価</th>
                <th style={thStyle}>小計</th>
                <th style={thStyle}>操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item, index) => (
                <tr key={index}>
                  {/* 商品ID入力欄 */}
                  <td style={tdStyle}>
                    <input
                      value={item.product_id}
                      onChange={(e) => updateItem(index, 'product_id', e.target.value)}
                      placeholder="商品ID"
                      style={{ width: '100%' }}
                    />
                    {errors[`items.${index}.product_id`] && (
                      <span style={errorStyle}>{errors[`items.${index}.product_id`]}</span>
                    )}
                  </td>
                  {/* 商品名入力欄 */}
                  <td style={tdStyle}>
                    <input
                      value={item.product_name}
                      onChange={(e) => updateItem(index, 'product_name', e.target.value)}
                      placeholder="商品名"
                      style={{ width: '100%' }}
                    />
                    {errors[`items.${index}.product_name`] && (
                      <span style={errorStyle}>{errors[`items.${index}.product_name`]}</span>
                    )}
                  </td>
                  {/* 数量入力欄 */}
                  <td style={tdStyle}>
                    <input
                      type="number"
                      value={item.quantity}
                      onChange={(e) => updateItem(index, 'quantity', Number(e.target.value))}
                      min={1}
                      style={{ width: '80px' }}
                    />
                    {errors[`items.${index}.quantity`] && (
                      <span style={errorStyle}>{errors[`items.${index}.quantity`]}</span>
                    )}
                  </td>
                  {/* 単価入力欄 */}
                  <td style={tdStyle}>
                    <input
                      type="number"
                      value={item.unit_price}
                      onChange={(e) => updateItem(index, 'unit_price', Number(e.target.value))}
                      min={0}
                      style={{ width: '100px' }}
                    />
                    {errors[`items.${index}.unit_price`] && (
                      <span style={errorStyle}>{errors[`items.${index}.unit_price`]}</span>
                    )}
                  </td>
                  {/* 小計（自動計算） */}
                  <td style={tdStyle}>{formatAmount(calcSubtotal(item))}</td>
                  {/* 削除ボタン */}
                  <td style={tdStyle}>
                    <button
                      type="button"
                      onClick={() => removeItem(index)}
                      disabled={items.length <= 1}
                      style={{ color: 'red' }}
                    >
                      削除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {/* アイテム追加ボタン */}
          <button type="button" onClick={addItem}>
            アイテムを追加
          </button>
        </div>

        {/* 合計金額表示 */}
        <div style={{ marginBottom: '16px', fontSize: '1.2em', fontWeight: 'bold' }}>
          合計: {formatAmount(calcTotal())}
        </div>

        {/* 備考入力欄 */}
        <div style={fieldStyle}>
          <label htmlFor="notes">備考</label>
          <textarea
            id="notes"
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            rows={3}
          />
        </div>

        {/* 送信・キャンセルボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={createOrder.isPending}>
            注文を作成
          </button>
          <button type="button" onClick={() => navigate({ to: '/' })}>
            キャンセル
          </button>
        </div>

        {/* APIエラー表示 */}
        {createOrder.error && (
          <p style={errorStyle}>
            注文の作成に失敗しました: {(createOrder.error as Error).message}
          </p>
        )}
      </form>
    </div>
  );
}

// フォームフィールドの共通スタイル
const fieldStyle: React.CSSProperties = {
  marginBottom: '12px',
  display: 'flex',
  flexDirection: 'column',
  gap: '4px',
};

// エラーメッセージのスタイル
const errorStyle: React.CSSProperties = {
  color: 'red',
  fontSize: '0.85em',
};

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
