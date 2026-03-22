import { z } from 'zod';

// BoardColumnエンティティのZodスキーマ: KanbanボードのカラムをWIP制限付きで表現
export const boardColumnSchema = z.object({
  id: z.string().uuid(),
  project_id: z.string().min(1, 'プロジェクトIDは必須です'),
  status_code: z.string().min(1, 'ステータスコードは必須です'),
  wip_limit: z.number().int().min(0, 'WIP制限は0以上にしてください'),
  task_count: z.number().int().min(0, 'タスク数は0以上にしてください'),
  version: z.number().int(),
  created_at: z.string(),
  updated_at: z.string(),
});

// カラムのタスク数をインクリメントする入力スキーマ
export const incrementColumnInputSchema = z.object({
  project_id: z.string().min(1, 'プロジェクトIDは必須です'),
  status_code: z.string().min(1, 'ステータスコードは必須です'),
});

// カラムのタスク数をデクリメントする入力スキーマ
export const decrementColumnInputSchema = z.object({
  project_id: z.string().min(1, 'プロジェクトIDは必須です'),
  status_code: z.string().min(1, 'ステータスコードは必須です'),
});

// WIP制限を更新する入力スキーマ
export const updateWipLimitInputSchema = z.object({
  wip_limit: z.number().int().min(0, 'WIP制限は0以上にしてください'),
});

// Zodスキーマから推論されたTypeScript型
export type BoardColumn = z.infer<typeof boardColumnSchema>;
export type IncrementColumnInput = z.infer<typeof incrementColumnInputSchema>;
export type DecrementColumnInput = z.infer<typeof decrementColumnInputSchema>;
export type UpdateWipLimitInput = z.infer<typeof updateWipLimitInputSchema>;
