import { z } from 'zod';

// 決済ステータスの列挙型スキーマ（サーバー契約に準拠）
export const paymentStatusSchema = z.enum(['initiated', 'completed', 'failed', 'refunded']);

// 決済エンティティのバリデーションスキーマ（サーバーレスポンス契約に準拠）
export const paymentSchema = z.object({
  id: z.string().uuid(),
  order_id: z.string().min(1, '注文IDは必須です'),
  customer_id: z.string().min(1, '顧客IDは必須です'),
  amount: z.number().min(0, '金額は0以上である必要があります'),
  currency: z.string().default('JPY'),
  status: paymentStatusSchema,
  // 決済方法: サーバー側ではenum→string(nullable)に変更
  payment_method: z.string().nullable().optional(),
  transaction_id: z.string().nullable().optional(),
  // エラー情報: failure_reason→error_code/error_messageに分離
  error_code: z.string().nullable().optional(),
  error_message: z.string().nullable().optional(),
  // 楽観的ロック用バージョン番号
  version: z.number(),
  created_at: z.string(),
  updated_at: z.string(),
});

// 決済開始リクエストのバリデーションスキーマ
export const initiatePaymentSchema = z.object({
  order_id: z.string().min(1, '注文IDは必須です'),
  customer_id: z.string().min(1, '顧客IDは必須です'),
  amount: z.number().min(1, '金額は1以上である必要があります'),
  currency: z.string().optional(),
  payment_method: z.string().optional(),
});

// Zodスキーマから推論されたTypeScript型
export type Payment = z.infer<typeof paymentSchema>;
export type PaymentStatus = z.infer<typeof paymentStatusSchema>;
export type InitiatePaymentInput = z.infer<typeof initiatePaymentSchema>;
