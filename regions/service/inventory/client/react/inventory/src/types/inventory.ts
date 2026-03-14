import { z } from 'zod';

// 在庫ステータスのEnum定義: 在庫あり・低在庫・在庫切れ
const inventoryStatusEnum = z.enum(['in_stock', 'low_stock', 'out_of_stock']);

// 在庫アイテムのZodスキーマ: 在庫データのバリデーションに使用
export const inventoryItemSchema = z.object({
  id: z.string().uuid(),
  product_id: z.string().min(1, '商品IDは必須です'),
  product_name: z.string().min(1, '商品名は必須です'),
  warehouse_id: z.string().min(1, '倉庫IDは必須です'),
  warehouse_name: z.string().min(1, '倉庫名は必須です'),
  quantity_available: z.number().int().min(0, '利用可能数は0以上です'),
  quantity_reserved: z.number().int().min(0, '予約数は0以上です'),
  reorder_point: z.number().int().min(0, '再注文点は0以上です'),
  status: inventoryStatusEnum,
  version: z.number().int(),
  created_at: z.string(),
  updated_at: z.string(),
});

// 在庫操作のZodスキーマ: 予約・解放リクエストのバリデーション
export const stockOperationSchema = z.object({
  product_id: z.string().min(1, '商品IDは必須です'),
  warehouse_id: z.string().min(1, '倉庫IDは必須です'),
  quantity: z.number().int().min(1, '数量は1以上です'),
});

// 在庫更新のZodスキーマ: 在庫数・再注文点の更新バリデーション
export const updateStockSchema = z.object({
  quantity_available: z.number().int().min(0, '利用可能数は0以上です'),
  reorder_point: z.number().int().min(0, '再注文点は0以上です').optional(),
});

// Zodスキーマから推論されたTypeScript型
export type InventoryItem = z.infer<typeof inventoryItemSchema>;
export type StockOperation = z.infer<typeof stockOperationSchema>;
export type UpdateStockInput = z.infer<typeof updateStockSchema>;
export type InventoryStatus = z.infer<typeof inventoryStatusEnum>;
