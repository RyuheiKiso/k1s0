import { useInventoryItem } from '../../hooks/useInventory';
import { InventoryForm } from './InventoryForm';
import type { InventoryStatus } from '../../types/inventory';

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
  if (error) return <div>エラーが発生しました: {(error as Error).message}</div>;

  // データが見つからない場合
  if (!item) return <div>在庫データが見つかりません。</div>;

  return (
    <div>
      <h1>在庫詳細</h1>

      {/* 在庫アイテムの基本情報 */}
      <div style={{ border: '1px solid #ccc', padding: '16px', marginBottom: '16px' }}>
        <h2>{item.product_name}</h2>
        <dl style={dlStyle}>
          <dt style={dtStyle}>商品ID</dt>
          <dd style={ddStyle}>{item.product_id}</dd>

          <dt style={dtStyle}>倉庫名</dt>
          <dd style={ddStyle}>{item.warehouse_name}</dd>

          <dt style={dtStyle}>倉庫ID</dt>
          <dd style={ddStyle}>{item.warehouse_id}</dd>

          <dt style={dtStyle}>利用可能数</dt>
          <dd style={ddStyle}>{item.quantity_available}</dd>

          <dt style={dtStyle}>予約数</dt>
          <dd style={ddStyle}>{item.quantity_reserved}</dd>

          <dt style={dtStyle}>再注文点</dt>
          <dd style={ddStyle}>{item.reorder_point}</dd>

          <dt style={dtStyle}>ステータス</dt>
          <dd style={ddStyle}>{statusLabels[item.status]}</dd>

          <dt style={dtStyle}>バージョン</dt>
          <dd style={ddStyle}>{item.version}</dd>

          <dt style={dtStyle}>更新日時</dt>
          <dd style={ddStyle}>{item.updated_at}</dd>
        </dl>
      </div>

      {/* 在庫操作フォーム（予約・解放） */}
      <InventoryForm
        productId={item.product_id}
        warehouseId={item.warehouse_id}
        inventoryId={id}
      />
    </div>
  );
}

// 定義リストのスタイル
const dlStyle: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: '150px 1fr',
  gap: '8px',
  margin: 0,
};

// 定義用語のスタイル
const dtStyle: React.CSSProperties = {
  fontWeight: 'bold',
  color: '#555',
};

// 定義説明のスタイル
const ddStyle: React.CSSProperties = {
  margin: 0,
};
