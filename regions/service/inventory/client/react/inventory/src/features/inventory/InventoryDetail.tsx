import { useInventoryItem } from '../../hooks/useInventory';
import { InventoryForm } from './InventoryForm';
import type { InventoryStatus } from '../../types/inventory';
import styles from './InventoryDetail.module.css';

// 詳細画面のProps: 在庫アイテムID
interface InventoryDetailProps {
  id: string;
}

// ステータスの日本語ラベルマッピング
const statusLabels: Record<InventoryStatus, string> = {
  in_stock: '在庫あり',
  low_stock: '低在庫',
  out_of_stock: '在庫切れ',
};

// 在庫詳細コンポーネント: アイテム情報表示と在庫操作フォームを提供
export function InventoryDetail({ id }: InventoryDetailProps) {
  const { data: item, isLoading, error } = useInventoryItem(id);

  // ローディング中の表示
  if (isLoading) return <div>読み込み中...</div>;

  // エラー発生時の表示
  if (error) return <div role="alert">エラーが発生しました: {(error as Error).message}</div>;

  // データが見つからない場合
  if (!item) return <div>在庫データが見つかりません。</div>;

  return (
    <main>
      <h1>在庫詳細</h1>

      {/* 在庫アイテムの基本情報 */}
      <section className={styles.section} aria-label="在庫基本情報">
        <h2>{item.product_name}</h2>
        <dl className={styles.dl}>
          <dt className={styles.dt}>商品ID</dt>
          <dd className={styles.dd}>{item.product_id}</dd>

          <dt className={styles.dt}>倉庫名</dt>
          <dd className={styles.dd}>{item.warehouse_name}</dd>

          <dt className={styles.dt}>倉庫ID</dt>
          <dd className={styles.dd}>{item.warehouse_id}</dd>

          <dt className={styles.dt}>利用可能数</dt>
          <dd className={styles.dd}>{item.quantity_available}</dd>

          <dt className={styles.dt}>予約数</dt>
          <dd className={styles.dd}>{item.quantity_reserved}</dd>

          <dt className={styles.dt}>再注文点</dt>
          <dd className={styles.dd}>{item.reorder_point}</dd>

          <dt className={styles.dt}>ステータス</dt>
          <dd className={styles.dd}>{statusLabels[item.status]}</dd>

          <dt className={styles.dt}>バージョン</dt>
          <dd className={styles.dd}>{item.version}</dd>

          <dt className={styles.dt}>更新日時</dt>
          <dd className={styles.dd}>{item.updated_at}</dd>
        </dl>
      </section>

      {/* 在庫操作フォーム（予約・解放） */}
      <InventoryForm
        productId={item.product_id}
        warehouseId={item.warehouse_id}
        inventoryId={id}
      />
    </main>
  );
}
