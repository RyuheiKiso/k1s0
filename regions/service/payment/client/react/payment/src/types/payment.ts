import { z } from 'zod';

// 決済ステータスの列挙型スキーマ
export const paymentStatusSchema = z.enum([
  'pending',
  'processing',
  'completed',
  'failed',
  'refunded',
]);

// 決済方法の列挙型スキーマ
export const paymentMethodSchema = z.enum([
  'credit_card',
  'bank_transfer',
  'convenience_store',
  'e_wallet',
]);

// 決済エンティティのZodスキーマ: 決済データのバリデーションに使用
export const paymentSchema = z.object({
  id: z.string().uuid(),
  order_id: z.string().min(1, '注文IDは必須です'),
  customer_id: z.string().min(1, '顧客IDは必須です'),
  amount: z.number().min(0, '金額は0以上である必要があります'),
  currency: z.string().default('JPY'),
  status: paymentStatusSchema,
  payment_method: paymentMethodSchema,
  transaction_id: z.string().nullable(),
  failure_reason: z.string().nullable(),
  refund_amount: z.number().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
});

// 決済開始時の入力スキーマ: 新規決済作成に必要なフィールド
export const initiatePaymentSchema = z.object({
  order_id: z.string().min(1, '注文IDは必須です'),
  customer_id: z.string().min(1, '顧客IDは必須です'),
  amount: z.number().min(1, '金額は1以上である必要があります'),
  currency: z.string().optional(),
  payment_method: paymentMethodSchema,
});

// Zodスキーマから推論されたTypeScript型
export type Payment = z.infer<typeof paymentSchema>;
export type PaymentStatus = z.infer<typeof paymentStatusSchema>;
export type PaymentMethod = z.infer<typeof paymentMethodSchema>;
export type InitiatePaymentInput = z.infer<typeof initiatePaymentSchema>;
