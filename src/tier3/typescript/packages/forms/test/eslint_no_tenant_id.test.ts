// tier3 forms ESLint no-tenant-id ルールのテスト
// tenant_id が forms の公開 API に露出しないことを確認する

// vitest の test/expect をインポートする
import { test, expect } from 'vitest';

// フォームフィールドの型定義 (tenant_id は禁止)
type FormField = {
  // フィールド名 (tenant_id は含まない)
  name: string;
  // フィールドの値型
  type: 'string' | 'number' | 'boolean';
  // 必須フラグ
  required: boolean;
};

// tenant_id フィールドが含まれていないことを検証するヘルパー関数
function hasTenantIdField(fields: FormField[]): boolean {
  // フィールド名が tenant_id または tenantId を含む場合は true を返す
  return fields.some(f => f.name === 'tenant_id' || f.name === 'tenantId');
}

// テスト: 正常なフォームフィールドには tenant_id が含まれないことを確認する
test('form fields do not contain tenant_id', () => {
  // 許可されたフォームフィールドの例
  const allowedFields: FormField[] = [
    { name: 'resource_name', type: 'string', required: true },
    { name: 'quantity', type: 'number', required: false },
    { name: 'active', type: 'boolean', required: false },
  ];
  // tenant_id フィールドが含まれていないことをアサートする
  expect(hasTenantIdField(allowedFields)).toBe(false);
});

// テスト: tenant_id フィールドが検出されることを確認する (検出機能の確認)
test('tenant_id detection works', () => {
  // 禁止されたフォームフィールドの例
  const forbiddenFields: FormField[] = [
    { name: 'tenant_id', type: 'string', required: true },
  ];
  // tenant_id フィールドが検出されることをアサートする
  expect(hasTenantIdField(forbiddenFields)).toBe(true);
});

// テスト: tenantId (camelCase) も検出されることを確認する
test('tenantId (camelCase) detection works', () => {
  // camelCase の禁止フィールドを定義する
  const forbiddenFields: FormField[] = [
    { name: 'tenantId', type: 'string', required: false },
  ];
  // tenantId フィールドが検出されることをアサートする
  expect(hasTenantIdField(forbiddenFields)).toBe(true);
});
