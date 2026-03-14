import { useState } from 'react';
import { useOrder, useUpdateOrderStatus } from '../../hooks/useOrders';
import type { OrderStatus } from '../../types/order';

// 注文詳細コンポーネントのProps
interface OrderDetailProps {
  orderId: string;
}

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

// 注文詳細コンポーネント: 注文情報とアイテム一覧、ステータス更新機能を提供
export function OrderDetail({ orderId }: OrderDetailProps) {
  const { data: order, isLoading, error } = useOrder(orderId);
  const updateStatus = useUpdateOrderStatus(orderId);

  // ステータス更新用の選択値
  const [newStatus, setNewStatus] = useState<OrderStatus | ''>('');

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

  // ステータス更新の実行
  const handleStatusUpdate = () => {
    if (!newStatus) return;
    updateStatus.mutate(
      { status: newStatus },
      {
        onSuccess: () => setNewStatus(''),
      }
    );
  };

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  // 注文データが存在しない場合
  if (!order) return <div>注文が見つかりませんでした。</div>;

  return (
    <div>
      <h1>注文詳細</h1>

      {/* ナビゲーションリンク */}
      <p>
        <a href="/">← 注文一覧に戻る</a>
      </p>

      {/* 注文基本情報 */}
      <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
        <h2>基本情報</h2>
        <table style={{ borderCollapse: 'collapse' }}>
          <tbody>
            <tr>
              <th style={infoThStyle}>注文ID</th>
              <td style={infoTdStyle}>{order.id}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>顧客ID</th>
              <td style={infoTdStyle}>{order.customer_id}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>ステータス</th>
              <td style={infoTdStyle}>
                {/* ステータスバッジ */}
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
            </tr>
            <tr>
              <th style={infoThStyle}>合計金額</th>
              <td style={infoTdStyle}>{formatAmount(order.total_amount, order.currency)}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>通貨</th>
              <td style={infoTdStyle}>{order.currency}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>備考</th>
              <td style={infoTdStyle}>{order.notes ?? '-'}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>バージョン</th>
              <td style={infoTdStyle}>{order.version}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>作成日時</th>
              <td style={infoTdStyle}>{formatDate(order.created_at)}</td>
            </tr>
            <tr>
              <th style={infoThStyle}>更新日時</th>
              <td style={infoTdStyle}>{formatDate(order.updated_at)}</td>
            </tr>
          </tbody>
        </table>
      </div>

      {/* ステータス更新セクション */}
      <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
        <h2>ステータス更新</h2>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <select
            value={newStatus}
            onChange={(e) => setNewStatus(e.target.value as OrderStatus | '')}
          >
            <option value="">選択してください</option>
            <option value="pending">保留中</option>
            <option value="confirmed">確認済</option>
            <option value="processing">処理中</option>
            <option value="shipped">発送済</option>
            <option value="delivered">配達済</option>
            <option value="cancelled">キャンセル</option>
          </select>
          <button
            onClick={handleStatusUpdate}
            disabled={!newStatus || updateStatus.isPending}
          >
            更新
          </button>
        </div>

        {/* ステータス更新エラー表示 */}
        {updateStatus.error && (
          <p style={{ color: 'red', fontSize: '0.85em' }}>
            ステータスの更新に失敗しました: {(updateStatus.error as Error).message}
          </p>
        )}
      </div>

      {/* 注文アイテム一覧テーブル */}
      <div style={{ border: '1px solid #ccc', padding: '16px' }}>
        <h2>注文アイテム</h2>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              <th style={thStyle}>商品ID</th>
              <th style={thStyle}>商品名</th>
              <th style={thStyle}>数量</th>
              <th style={thStyle}>単価</th>
              <th style={thStyle}>小計</th>
            </tr>
          </thead>
          <tbody>
            {order.items.map((item, index) => (
              <tr key={index}>
                <td style={tdStyle}>{item.product_id}</td>
                <td style={tdStyle}>{item.product_name}</td>
                <td style={tdStyle}>{item.quantity}</td>
                <td style={tdStyle}>{formatAmount(item.unit_price, order.currency)}</td>
                <td style={tdStyle}>{formatAmount(item.subtotal, order.currency)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// 情報テーブルのヘッダーセルスタイル
const infoThStyle: React.CSSProperties = {
  padding: '6px 16px 6px 0',
  textAlign: 'left',
  whiteSpace: 'nowrap',
  verticalAlign: 'top',
};

// 情報テーブルのデータセルスタイル
const infoTdStyle: React.CSSProperties = {
  padding: '6px 0',
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
