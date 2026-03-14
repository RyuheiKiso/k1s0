import { z } from 'zod';

// 注文ステータスの列挙型スキーマ
export const orderStatusSchema = z.enum([
  'pending',
  'confirmed',
  'processing',
  'shipped',
  'delivered',
  'cancelled',
]);

// 注文アイテムのZodスキーマ: 注文内の各商品行を表現
export const orderItemSchema = z.object({
  product_id: z.string().min(1, '商品IDは必須です'),
  product_name: z.string().min(1, '商品名は必須です'),
  quantity: z.number().int().min(1, '数量は1以上にしてください'),
  unit_price: z.number().min(0, '単価は0以上にしてください'),
  subtotal: z.number().min(0, '小計は0以上にしてください'),
});

// 注文エンティティのZodスキーマ: 注文の全フィールドをバリデーション
export const orderSchema = z.object({
  id: z.string().uuid(),
  customer_id: z.string().min(1, '顧客IDは必須です'),
  status: orderStatusSchema,
  total_amount: z.number().min(0),
  currency: z.string().default('JPY'),
  items: z.array(orderItemSchema),
  notes: z.string().nullable(),
  version: z.number().int(),
  created_at: z.string(),
  updated_at: z.string(),
});

// 注文作成時の入力スキーマ（ID・タイムスタンプ・計算フィールドを除く）
export const createOrderInputSchema = z.object({
  customer_id: z.string().min(1, '顧客IDは必須です'),
  currency: z.string().optional(),
  items: z
    .array(
      z.object({
        product_id: z.string().min(1, '商品IDは必須です'),
        product_name: z.string().min(1, '商品名は必須です'),
        quantity: z.number().int().min(1, '数量は1以上にしてください'),
        unit_price: z.number().min(0, '単価は0以上にしてください'),
      })
    )
    .min(1, '注文アイテムは1つ以上必要です'),
  notes: z.string().optional(),
});

// 注文ステータス更新の入力スキーマ
export const updateOrderStatusInputSchema = z.object({
  status: orderStatusSchema,
});

// Zodスキーマから推論されたTypeScript型
export type OrderStatus = z.infer<typeof orderStatusSchema>;
export type OrderItem = z.infer<typeof orderItemSchema>;
export type Order = z.infer<typeof orderSchema>;
export type CreateOrderInput = z.infer<typeof createOrderInputSchema>;
export type UpdateOrderStatusInput = z.infer<typeof updateOrderStatusInputSchema>;
