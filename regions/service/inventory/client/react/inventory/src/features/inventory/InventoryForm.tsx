import { useState } from 'react';
import { stockOperationSchema, updateStockSchema } from '../../types/inventory';
import { useReserveStock, useReleaseStock, useUpdateStock } from '../../hooks/useInventory';

// 在庫操作フォームのProps
interface InventoryFormProps {
  productId: string;
  warehouseId: string;
  inventoryId: string;
}

// 操作タイプの定義
type OperationType = 'reserve' | 'release' | 'update';

// 在庫操作フォームコンポーネント: 予約・解放・更新操作をZodバリデーション付きで提供
export function InventoryForm({ productId, warehouseId, inventoryId }: InventoryFormProps) {
  // 操作タイプの状態管理
  const [operationType, setOperationType] = useState<OperationType>('reserve');
  // 数量の入力値
  const [quantity, setQuantity] = useState(0);
  // 在庫更新用の利用可能数
  const [quantityAvailable, setQuantityAvailable] = useState(0);
  // 在庫更新用の再注文点
  const [reorderPoint, setReorderPoint] = useState<number | undefined>(undefined);
  // バリデーションエラーメッセージ
  const [errors, setErrors] = useState<Record<string, string>>({});

  const reserveStock = useReserveStock();
  const releaseStock = useReleaseStock();
  const updateStock = useUpdateStock(inventoryId);

  // フォーム送信時のバリデーションとAPI呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    if (operationType === 'reserve' || operationType === 'release') {
      // 予約・解放操作のバリデーション
      const input = {
        product_id: productId,
        warehouse_id: warehouseId,
        quantity,
      };

      const result = stockOperationSchema.safeParse(input);
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

      // 操作タイプに応じたミューテーション実行
      const mutation = operationType === 'reserve' ? reserveStock : releaseStock;
      mutation.mutate(result.data, {
        onSuccess: () => {
          setQuantity(0);
        },
      });
    } else {
      // 在庫更新のバリデーション
      const input = {
        quantity_available: quantityAvailable,
        reorder_point: reorderPoint,
      };

      const result = updateStockSchema.safeParse(input);
      if (!result.success) {
        const fieldErrors: Record<string, string> = {};
        result.error.errors.forEach((err) => {
          const field = err.path.join('.');
          fieldErrors[field] = err.message;
        });
        setErrors(fieldErrors);
        return;
      }

      updateStock.mutate(result.data, {
        onSuccess: () => {
          setQuantityAvailable(0);
          setReorderPoint(undefined);
        },
      });
    }
  };

  // ミューテーション実行中かどうか
  const isPending = reserveStock.isPending || releaseStock.isPending || updateStock.isPending;
  // ミューテーションエラー
  const mutationError = reserveStock.error || releaseStock.error || updateStock.error;

  return (
    <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
      <h2>在庫操作</h2>

      {/* 操作タイプ選択 */}
      <div style={{ marginBottom: '16px', display: 'flex', gap: '8px' }}>
        <button
          type="button"
          onClick={() => setOperationType('reserve')}
          style={operationType === 'reserve' ? activeTabStyle : tabStyle}
        >
          在庫予約
        </button>
        <button
          type="button"
          onClick={() => setOperationType('release')}
          style={operationType === 'release' ? activeTabStyle : tabStyle}
        >
          予約解放
        </button>
        <button
          type="button"
          onClick={() => setOperationType('update')}
          style={operationType === 'update' ? activeTabStyle : tabStyle}
        >
          在庫更新
        </button>
      </div>

      <form onSubmit={handleSubmit}>
        {/* 予約・解放操作の数量入力欄 */}
        {(operationType === 'reserve' || operationType === 'release') && (
          <div style={fieldStyle}>
            <label htmlFor="quantity">数量</label>
            <input
              id="quantity"
              type="number"
              value={quantity}
              onChange={(e) => setQuantity(Number(e.target.value))}
              min={1}
              required
            />
            {errors.quantity && <span style={errorStyle}>{errors.quantity}</span>}
          </div>
        )}

        {/* 在庫更新の入力欄 */}
        {operationType === 'update' && (
          <>
            {/* 利用可能数入力 */}
            <div style={fieldStyle}>
              <label htmlFor="quantity_available">利用可能数</label>
              <input
                id="quantity_available"
                type="number"
                value={quantityAvailable}
                onChange={(e) => setQuantityAvailable(Number(e.target.value))}
                min={0}
                required
              />
              {errors.quantity_available && (
                <span style={errorStyle}>{errors.quantity_available}</span>
              )}
            </div>

            {/* 再注文点入力（任意） */}
            <div style={fieldStyle}>
              <label htmlFor="reorder_point">再注文点（任意）</label>
              <input
                id="reorder_point"
                type="number"
                value={reorderPoint ?? ''}
                onChange={(e) =>
                  setReorderPoint(e.target.value ? Number(e.target.value) : undefined)
                }
                min={0}
              />
              {errors.reorder_point && <span style={errorStyle}>{errors.reorder_point}</span>}
            </div>
          </>
        )}

        {/* 送信ボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={isPending}>
            {operationType === 'reserve'
              ? '予約する'
              : operationType === 'release'
                ? '解放する'
                : '更新する'}
          </button>
        </div>

        {/* API エラー表示 */}
        {mutationError && (
          <p style={errorStyle}>操作に失敗しました: {(mutationError as Error).message}</p>
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

// タブボタンのスタイル
const tabStyle: React.CSSProperties = {
  padding: '8px 16px',
  border: '1px solid #ccc',
  backgroundColor: '#fff',
  cursor: 'pointer',
};

// アクティブタブのスタイル
const activeTabStyle: React.CSSProperties = {
  padding: '8px 16px',
  border: '1px solid #007bff',
  backgroundColor: '#007bff',
  color: '#fff',
  cursor: 'pointer',
};
