import { z } from 'zod';

// アクティビティステータスの列挙型スキーマ
// active: アクティブ、submitted: 承認申請済み、approved: 承認済み、rejected: 却下済み
export const activityStatusSchema = z.enum([
  'active',
  'submitted',
  'approved',
  'rejected',
]);

// アクティビティ種別の列挙型スキーマ
// comment: コメント、time_entry: 作業時間記録、status_change: ステータス変更、assignment: 担当割当
export const activityTypeSchema = z.enum([
  'comment',
  'time_entry',
  'status_change',
  'assignment',
]);

// アクティビティエンティティのZodスキーマ: 全フィールドをバリデーション
export const activitySchema = z.object({
  id: z.string().uuid(),
  task_id: z.string().min(1, 'タスクIDは必須です'),
  actor_id: z.string().min(1, 'アクターIDは必須です'),
  activity_type: activityTypeSchema,
  content: z.string().nullable(),
  duration_minutes: z.number().int().nullable(),
  status: activityStatusSchema,
  metadata: z.record(z.unknown()).nullable(),
  idempotency_key: z.string().nullable(),
  version: z.number().int(),
  created_at: z.string(),
  updated_at: z.string(),
});

// アクティビティ作成時の入力スキーマ（ID・タイムスタンプ・計算フィールドを除く）
export const createActivityInputSchema = z.object({
  task_id: z.string().min(1, 'タスクIDは必須です'),
  actor_id: z.string().min(1, 'アクターIDは必須です'),
  activity_type: activityTypeSchema,
  content: z.string().optional(),
  duration_minutes: z.number().int().min(0, '作業時間は0以上にしてください').optional(),
  metadata: z.record(z.unknown()).optional(),
  idempotency_key: z.string().optional(),
});

// 承認申請入力スキーマ（ボディ不要）
export const submitActivityInputSchema = z.object({});

// 承認入力スキーマ（ボディ不要）
export const approveActivityInputSchema = z.object({});

// 却下入力スキーマ（却下理由は任意）
export const rejectActivityInputSchema = z.object({
  reason: z.string().optional(),
});

// Zodスキーマから推論されたTypeScript型
export type ActivityStatus = z.infer<typeof activityStatusSchema>;
export type ActivityType = z.infer<typeof activityTypeSchema>;
export type Activity = z.infer<typeof activitySchema>;
export type CreateActivityInput = z.infer<typeof createActivityInputSchema>;
export type SubmitActivityInput = z.infer<typeof submitActivityInputSchema>;
export type ApproveActivityInput = z.infer<typeof approveActivityInputSchema>;
export type RejectActivityInput = z.infer<typeof rejectActivityInputSchema>;
