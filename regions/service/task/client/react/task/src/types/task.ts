import { z } from 'zod';

// タスクステータスの列挙型スキーマ
export const taskStatusSchema = z.enum([
  'open',
  'in_progress',
  'review',
  'done',
  'cancelled',
]);

// タスク優先度の列挙型スキーマ
export const taskPrioritySchema = z.enum([
  'low',
  'medium',
  'high',
  'critical',
]);

// タスクエンティティのZodスキーマ: タスクの全フィールドをバリデーション
export const taskSchema = z.object({
  id: z.string().uuid(),
  project_id: z.string().min(1, 'プロジェクトIDは必須です'),
  title: z.string().min(1, 'タイトルは必須です'),
  description: z.string().nullable(),
  status: taskStatusSchema,
  priority: taskPrioritySchema,
  assignee_id: z.string().nullable(),
  reporter_id: z.string().min(1, '報告者IDは必須です'),
  due_date: z.string().nullable(),
  labels: z.array(z.string()),
  created_by: z.string().min(1, '作成者IDは必須です'),
  updated_by: z.string().min(1, '更新者IDは必須です'),
  version: z.number().int(),
  created_at: z.string(),
  updated_at: z.string(),
});

// タスク作成時の入力スキーマ（ID・タイムスタンプ等を除く）
export const createTaskInputSchema = z.object({
  project_id: z.string().min(1, 'プロジェクトIDは必須です'),
  title: z.string().min(1, 'タイトルは必須です'),
  description: z.string().optional(),
  priority: taskPrioritySchema.optional(),
  assignee_id: z.string().optional(),
  due_date: z.string().optional(),
  labels: z.array(z.string()).optional(),
});

// タスクステータス更新の入力スキーマ
export const updateTaskStatusInputSchema = z.object({
  status: taskStatusSchema,
});

// Zodスキーマから推論されたTypeScript型
export type TaskStatus = z.infer<typeof taskStatusSchema>;
export type TaskPriority = z.infer<typeof taskPrioritySchema>;
export type Task = z.infer<typeof taskSchema>;
export type CreateTaskInput = z.infer<typeof createTaskInputSchema>;
export type UpdateTaskStatusInput = z.infer<typeof updateTaskStatusInputSchema>;
