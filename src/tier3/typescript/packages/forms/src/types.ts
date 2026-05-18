// tier3 forms package の型定義
// tenant_id を含まない業務フォームフィールドの型

// FormFieldBase: 1 フォームフィールドの型定義
export type FormFieldBase = {
  // フィールド名 (tenant_id / tenantId は含まない)
  name: string;
  // フィールドの値型
  type: 'string' | 'number' | 'boolean' | 'date';
  // 必須フラグ
  required: boolean;
  // フィールドの表示ラベル
  label?: string;
};
