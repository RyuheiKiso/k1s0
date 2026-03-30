import { z } from 'zod';

// プロジェクトタイプのZodスキーマ: プロジェクトタイプの作成・編集時のバリデーションに使用
// max値はproto buf のバリデーションルールに準拠する（tenant.proto参照）
export const projectTypeSchema = z.object({
  id: z.string().uuid(),
  code: z.string().min(1, 'コードは必須です').max(64, 'コードは64文字以内で入力してください'),
  display_name: z.string().min(1, '表示名は必須です').max(128, '表示名は128文字以内で入力してください'),
  description: z.string().max(1024, '説明は1024文字以内で入力してください').nullable(),
  default_workflow: z.string().max(256, 'デフォルトワークフローは256文字以内で入力してください').nullable(),
  is_active: z.boolean(),
  sort_order: z.number().int().min(0),
  created_by: z.string().max(256, '作成者は256文字以内で入力してください'),
  created_at: z.string(),
  updated_at: z.string(),
});

// プロジェクトタイプ作成時の入力スキーマ（ID・タイムスタンプを除く）
// max値はproto buf のバリデーションルールに準拠する（tenant.proto参照）
export const createProjectTypeSchema = z.object({
  code: z.string().min(1, 'コードは必須です').max(64, 'コードは64文字以内で入力してください'),
  display_name: z.string().min(1, '表示名は必須です').max(128, '表示名は128文字以内で入力してください'),
  description: z.string().max(1024, '説明は1024文字以内で入力してください').nullable().optional(),
  default_workflow: z.string().max(256, 'デフォルトワークフローは256文字以内で入力してください').nullable().optional(),
  is_active: z.boolean().default(true),
  sort_order: z.number().int().min(0).default(0),
});

// プロジェクトタイプ更新時の入力スキーマ
export const updateProjectTypeSchema = createProjectTypeSchema.partial();

// ステータス定義のZodスキーマ: ステータス定義のバリデーションに使用
// max値はproto buf のバリデーションルールに準拠する（tenant.proto参照）
export const statusDefinitionSchema = z.object({
  id: z.string().uuid(),
  project_type_id: z.string().uuid(),
  code: z.string().min(1, 'コードは必須です').max(64, 'コードは64文字以内で入力してください'),
  display_name: z.string().min(1, '表示名は必須です').max(128, '表示名は128文字以内で入力してください'),
  description: z.string().max(1024, '説明は1024文字以内で入力してください').nullable(),
  color: z.string().nullable(),
  allowed_transitions: z.array(z.string()).nullable(),
  is_initial: z.boolean(),
  is_terminal: z.boolean(),
  sort_order: z.number().int().min(0),
  created_by: z.string().max(256, '作成者は256文字以内で入力してください'),
  created_at: z.string(),
  updated_at: z.string(),
});

// ステータス定義作成時の入力スキーマ
// max値はproto buf のバリデーションルールに準拠する（tenant.proto参照）
export const createStatusDefinitionSchema = z.object({
  code: z.string().min(1, 'コードは必須です').max(64, 'コードは64文字以内で入力してください'),
  display_name: z.string().min(1, '表示名は必須です').max(128, '表示名は128文字以内で入力してください'),
  description: z.string().max(1024, '説明は1024文字以内で入力してください').nullable().optional(),
  color: z.string().nullable().optional(),
  allowed_transitions: z.array(z.string()).nullable().optional(),
  is_initial: z.boolean().default(false),
  is_terminal: z.boolean().default(false),
  sort_order: z.number().int().min(0).default(0),
});

// ステータス定義更新時の入力スキーマ
export const updateStatusDefinitionSchema = createStatusDefinitionSchema.partial();

// ステータス定義バージョンのZodスキーマ: バージョン履歴の型定義
// max値はproto buf のバリデーションルールに準拠する（tenant.proto参照）
export const statusDefinitionVersionSchema = z.object({
  id: z.string().uuid(),
  status_definition_id: z.string().uuid(),
  version_number: z.number().int(),
  before_data: z.record(z.string(), z.unknown()).nullable(),
  after_data: z.record(z.string(), z.unknown()),
  changed_by: z.string().max(256, '変更者は256文字以内で入力してください'),
  change_reason: z.string().max(512, '変更理由は512文字以内で入力してください').nullable(),
  created_at: z.string(),
});

// テナントプロジェクト拡張のZodスキーマ: テナント固有のカスタマイズ定義
// tenant_id の max 36 は UUID フォーマットに対応する（tenant.proto参照）
export const tenantProjectExtensionSchema = z.object({
  id: z.string().uuid(),
  tenant_id: z.string().max(36, 'テナントIDは36文字以内で入力してください'),
  status_definition_id: z.string().uuid(),
  display_name_override: z.string().max(128, '表示名オーバーライドは128文字以内で入力してください').nullable(),
  attributes_override: z.record(z.string(), z.unknown()).nullable(),
  is_enabled: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
});

// テナント拡張の更新入力スキーマ
export const updateTenantExtensionSchema = z.object({
  display_name_override: z.string().nullable().optional(),
  attributes_override: z.record(z.string(), z.unknown()).nullable().optional(),
  is_enabled: z.boolean().optional(),
});

// Zodスキーマから推論されたTypeScript型
export type ProjectType = z.infer<typeof projectTypeSchema>;
export type CreateProjectTypeInput = z.infer<typeof createProjectTypeSchema>;
export type UpdateProjectTypeInput = z.infer<typeof updateProjectTypeSchema>;
export type StatusDefinition = z.infer<typeof statusDefinitionSchema>;
export type CreateStatusDefinitionInput = z.infer<typeof createStatusDefinitionSchema>;
export type UpdateStatusDefinitionInput = z.infer<typeof updateStatusDefinitionSchema>;
export type StatusDefinitionVersion = z.infer<typeof statusDefinitionVersionSchema>;
export type TenantProjectExtension = z.infer<typeof tenantProjectExtensionSchema>;
export type UpdateTenantExtensionInput = z.infer<typeof updateTenantExtensionSchema>;
