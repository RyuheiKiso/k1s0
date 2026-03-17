import { useState } from 'react';
import { useOrder, useUpdateOrderStatus } from '../../hooks/useOrders';
import type { OrderStatus } from '../../types/order';
import styles from './OrderDetail.module.css';

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

// ステータスバッジのCSSクラス名マッピング
const statusClassMap: Record<OrderStatus, string> = {
  pending: 'statusPending',
  confirmed: 'statusConfirmed',
  processing: 'statusProcessing',
  shipped: 'statusShipped',
  delivered: 'statusDelivered',
  cancelled: 'statusCancelled',
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
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // 注文データが存在しない場合
  if (!order) return <div>注文が見つかりませんでした。</div>;

  return (
    <main>
      <h1>注文詳細</h1>

      {/* ナビゲーションリンク */}
      <nav aria-label="パンくずナビゲーション">
        <a href="/">← 注文一覧に戻る</a>
      </nav>

      {/* 注文基本情報 */}
      <section className={styles.section} aria-label="注文基本情報">
        <h2>基本情報</h2>
        <table style={{ borderCollapse: 'collapse' }}>
          <tbody>
            <tr>
              <th className={styles.infoTh}>注文ID</th>
              <td className={styles.infoTd}>{order.id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>顧客ID</th>
              <td className={styles.infoTd}>{order.customer_id}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>ステータス</th>
              <td className={styles.infoTd}>
                {/* ステータスバッジ */}
                <span className={`${styles.statusBadge} ${styles[statusClassMap[order.status]]}`}>
                  {statusLabels[order.status]}
                </span>
              </td>
            </tr>
            <tr>
              <th className={styles.infoTh}>合計金額</th>
              <td className={styles.infoTd}>{formatAmount(order.total_amount, order.currency)}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>通貨</th>
              <td className={styles.infoTd}>{order.currency}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>備考</th>
              <td className={styles.infoTd}>{order.notes ?? '-'}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>バージョン</th>
              <td className={styles.infoTd}>{order.version}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>作成日時</th>
              <td className={styles.infoTd}>{formatDate(order.created_at)}</td>
            </tr>
            <tr>
              <th className={styles.infoTh}>更新日時</th>
              <td className={styles.infoTd}>{formatDate(order.updated_at)}</td>
            </tr>
          </tbody>
        </table>
      </section>

      {/* ステータス更新セクション */}
      <section className={styles.section} aria-label="ステータス更新">
        <h2>ステータス更新</h2>
        <div className={styles.statusUpdateControls}>
          <label htmlFor="new-status" className="sr-only">新しいステータス</label>
          <select
            id="new-status"
            value={newStatus}
            onChange={(e) => setNewStatus(e.target.value as OrderStatus | '')}
            aria-label="新しいステータスを選択"
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
            aria-label="ステータスを更新"
          >
            更新
          </button>
        </div>

        {/* ステータス更新エラー表示 */}
        {updateStatus.error && (
          <p className={styles.error} role="alert">
            ステータスの更新に失敗しました: {(updateStatus.error as Error).message}
          </p>
        )}
      </section>

      {/* 注文アイテム一覧テーブル */}
      <section className={styles.section} aria-label="注文アイテム">
        <h2>注文アイテム</h2>
        <table className={styles.table} aria-label="注文アイテム一覧">
          <thead>
            <tr>
              <th className={styles.th}>商品ID</th>
              <th className={styles.th}>商品名</th>
              <th className={styles.th}>数量</th>
              <th className={styles.th}>単価</th>
              <th className={styles.th}>小計</th>
            </tr>
          </thead>
          <tbody>
            {order.items.map((item, index) => (
              <tr key={index}>
                <td className={styles.td}>{item.product_id}</td>
                <td className={styles.td}>{item.product_name}</td>
                <td className={styles.td}>{item.quantity}</td>
                <td className={styles.td}>{formatAmount(item.unit_price, order.currency)}</td>
                <td className={styles.td}>{formatAmount(item.subtotal, order.currency)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </main>
  );
}
