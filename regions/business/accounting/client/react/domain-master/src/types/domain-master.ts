import { z } from 'zod';

// マスタカテゴリのZodスキーマ: カテゴリの作成・編集時のバリデーションに使用
export const masterCategorySchema = z.object({
  id: z.string().uuid(),
  code: z.string().min(1, 'コードは必須です'),
  display_name: z.string().min(1, '表示名は必須です'),
  description: z.string().nullable(),
  validation_schema: z.record(z.string(), z.unknown()).nullable(),
  is_active: z.boolean(),
  sort_order: z.number().int().min(0),
  created_by: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
});

// カテゴリ作成時の入力スキーマ（ID・タイムスタンプを除く）
export const createCategorySchema = z.object({
  code: z.string().min(1, 'コードは必須です'),
  display_name: z.string().min(1, '表示名は必須です'),
  description: z.string().nullable().optional(),
  validation_schema: z.record(z.string(), z.unknown()).nullable().optional(),
  is_active: z.boolean().default(true),
  sort_order: z.number().int().min(0).default(0),
});

// カテゴリ更新時の入力スキーマ
export const updateCategorySchema = createCategorySchema.partial();

// マスタアイテムのZodスキーマ: アイテムのバリデーションに使用
export const masterItemSchema = z.object({
  id: z.string().uuid(),
  category_id: z.string().uuid(),
  code: z.string().min(1, 'コードは必須です'),
  display_name: z.string().min(1, '表示名は必須です'),
  description: z.string().nullable(),
  attributes: z.record(z.string(), z.unknown()).nullable(),
  parent_item_id: z.string().uuid().nullable(),
  effective_from: z.string().nullable(),
  effective_until: z.string().nullable(),
  is_active: z.boolean(),
  sort_order: z.number().int().min(0),
  created_by: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
});

// アイテム作成時の入力スキーマ
export const createItemSchema = z.object({
  code: z.string().min(1, 'コードは必須です'),
  display_name: z.string().min(1, '表示名は必須です'),
  description: z.string().nullable().optional(),
  attributes: z.record(z.string(), z.unknown()).nullable().optional(),
  parent_item_id: z.string().uuid().nullable().optional(),
  effective_from: z.string().nullable().optional(),
  effective_until: z.string().nullable().optional(),
  is_active: z.boolean().default(true),
  sort_order: z.number().int().min(0).default(0),
});

// アイテム更新時の入力スキーマ
export const updateItemSchema = createItemSchema.partial();

// マスタアイテムバージョンのZodスキーマ: 変更履歴の型定義
export const masterItemVersionSchema = z.object({
  id: z.string().uuid(),
  item_id: z.string().uuid(),
  version_number: z.number().int(),
  before_data: z.record(z.string(), z.unknown()).nullable(),
  after_data: z.record(z.string(), z.unknown()),
  changed_by: z.string(),
  change_reason: z.string().nullable(),
  created_at: z.string(),
});

// テナントマスタ拡張のZodスキーマ: テナント固有のカスタマイズ定義
export const tenantMasterExtensionSchema = z.object({
  id: z.string().uuid(),
  tenant_id: z.string(),
  item_id: z.string().uuid(),
  display_name_override: z.string().nullable(),
  attributes_override: z.record(z.string(), z.unknown()).nullable(),
  created_at: z.string(),
  updated_at: z.string(),
});

// テナント拡張の更新入力スキーマ
export const updateTenantExtensionSchema = z.object({
  display_name_override: z.string().nullable().optional(),
  attributes_override: z.record(z.string(), z.unknown()).nullable().optional(),
});

// Zodスキーマから推論されたTypeScript型
export type MasterCategory = z.infer<typeof masterCategorySchema>;
export type CreateCategoryInput = z.infer<typeof createCategorySchema>;
export type UpdateCategoryInput = z.infer<typeof updateCategorySchema>;
export type MasterItem = z.infer<typeof masterItemSchema>;
export type CreateItemInput = z.infer<typeof createItemSchema>;
export type UpdateItemInput = z.infer<typeof updateItemSchema>;
export type MasterItemVersion = z.infer<typeof masterItemVersionSchema>;
export type TenantMasterExtension = z.infer<typeof tenantMasterExtensionSchema>;
export type UpdateTenantExtensionInput = z.infer<typeof updateTenantExtensionSchema>;
