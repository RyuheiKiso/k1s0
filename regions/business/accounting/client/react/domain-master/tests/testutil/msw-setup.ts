import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type {
  MasterCategory,
  MasterItem,
  MasterItemVersion,
  TenantMasterExtension,
} from '../../src/types/domain-master';

// テスト用モックデータ: カテゴリ
const mockCategories: MasterCategory[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    code: 'DEPT',
    display_name: '部門',
    description: '組織の部門マスタ',
    validation_schema: null,
    is_active: true,
    sort_order: 1,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    code: 'ACCT',
    display_name: '勘定科目',
    description: '会計の勘定科目マスタ',
    validation_schema: null,
    is_active: true,
    sort_order: 2,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: アイテム
const mockItems: MasterItem[] = [
  {
    id: '660e8400-e29b-41d4-a716-446655440001',
    category_id: '550e8400-e29b-41d4-a716-446655440001',
    code: 'SALES',
    display_name: '営業部',
    description: '営業部門',
    attributes: null,
    parent_item_id: null,
    effective_from: '2024-01-01',
    effective_until: null,
    is_active: true,
    sort_order: 1,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: バージョン履歴
const mockVersions: MasterItemVersion[] = [
  {
    id: '770e8400-e29b-41d4-a716-446655440001',
    item_id: '660e8400-e29b-41d4-a716-446655440001',
    version_number: 1,
    before_data: null,
    after_data: { display_name: '営業部', code: 'SALES' },
    changed_by: 'admin',
    change_reason: '初期作成',
    created_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: テナント拡張
const mockTenantExtension: TenantMasterExtension = {
  id: '880e8400-e29b-41d4-a716-446655440001',
  tenant_id: 'tenant-001',
  item_id: '660e8400-e29b-41d4-a716-446655440001',
  display_name_override: '営業本部',
  attributes_override: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // カテゴリ一覧取得
  http.get('/bff/api/v1/categories', () => {
    return HttpResponse.json({ categories: mockCategories });
  }),

  // カテゴリ作成
  http.post('/bff/api/v1/categories', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newCategory: MasterCategory = {
      id: crypto.randomUUID(),
      code: body.code as string,
      display_name: body.display_name as string,
      description: (body.description as string) ?? null,
      validation_schema: null,
      is_active: (body.is_active as boolean) ?? true,
      sort_order: (body.sort_order as number) ?? 0,
      created_by: 'test-user',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newCategory, { status: 201 });
  }),

  // カテゴリ個別取得
  http.get('/bff/api/v1/categories/:code', ({ params }) => {
    const category = mockCategories.find((c) => c.code === params.code);
    if (!category) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(category);
  }),

  // カテゴリ更新
  http.put('/bff/api/v1/categories/:code', async ({ params, request }) => {
    const category = mockCategories.find((c) => c.code === params.code);
    if (!category) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({ ...category, ...body, updated_at: new Date().toISOString() });
  }),

  // カテゴリ削除
  http.delete('/bff/api/v1/categories/:code', () => {
    return new HttpResponse(null, { status: 204 });
  }),

  // アイテム一覧取得
  http.get('/bff/api/v1/categories/:code/items', () => {
    return HttpResponse.json({ items: mockItems });
  }),

  // アイテム作成
  http.post('/bff/api/v1/categories/:code/items', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newItem: MasterItem = {
      id: crypto.randomUUID(),
      category_id: '550e8400-e29b-41d4-a716-446655440001',
      code: body.code as string,
      display_name: body.display_name as string,
      description: (body.description as string) ?? null,
      attributes: null,
      parent_item_id: (body.parent_item_id as string) ?? null,
      effective_from: (body.effective_from as string) ?? null,
      effective_until: (body.effective_until as string) ?? null,
      is_active: (body.is_active as boolean) ?? true,
      sort_order: (body.sort_order as number) ?? 0,
      created_by: 'test-user',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newItem, { status: 201 });
  }),

  // アイテム個別取得
  http.get('/bff/api/v1/categories/:code/items/:itemCode', ({ params }) => {
    const item = mockItems.find((i) => i.code === params.itemCode);
    if (!item) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(item);
  }),

  // アイテム更新
  http.put('/bff/api/v1/categories/:code/items/:itemCode', async ({ params, request }) => {
    const item = mockItems.find((i) => i.code === params.itemCode);
    if (!item) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({ ...item, ...body, updated_at: new Date().toISOString() });
  }),

  // アイテム削除
  http.delete('/bff/api/v1/categories/:code/items/:itemCode', () => {
    return new HttpResponse(null, { status: 204 });
  }),

  // バージョン履歴取得
  http.get('/bff/api/v1/categories/:code/items/:itemCode/versions', () => {
    return HttpResponse.json({ versions: mockVersions });
  }),

  // テナント拡張取得
  http.get('/bff/api/v1/tenants/:tenantId/items/:itemId', () => {
    return HttpResponse.json(mockTenantExtension);
  }),

  // テナント拡張更新
  http.put('/bff/api/v1/tenants/:tenantId/items/:itemId', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({ ...mockTenantExtension, ...body, updated_at: new Date().toISOString() });
  }),

  // テナント拡張削除
  http.delete('/bff/api/v1/tenants/:tenantId/items/:itemId', () => {
    return new HttpResponse(null, { status: 204 });
  }),

  // テナント別カテゴリアイテム一覧
  http.get('/bff/api/v1/tenants/:tenantId/categories/:code/items', () => {
    return HttpResponse.json({ items: mockItems });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockCategories, mockItems, mockVersions, mockTenantExtension };
