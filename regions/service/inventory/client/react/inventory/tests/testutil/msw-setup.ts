import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { InventoryItem } from '../../src/types/inventory';

// テスト用モックデータ: 在庫アイテム
const mockInventoryItems: InventoryItem[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    product_id: 'PROD-001',
    product_name: 'ノートパソコン',
    warehouse_id: 'WH-001',
    warehouse_name: '東京倉庫',
    quantity_available: 150,
    quantity_reserved: 20,
    reorder_point: 30,
    status: 'in_stock',
    version: 1,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    product_id: 'PROD-002',
    product_name: 'モニター',
    warehouse_id: 'WH-001',
    warehouse_name: '東京倉庫',
    quantity_available: 5,
    quantity_reserved: 3,
    reorder_point: 10,
    status: 'low_stock',
    version: 2,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-02-01T00:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440003',
    product_id: 'PROD-003',
    product_name: 'キーボード',
    warehouse_id: 'WH-002',
    warehouse_name: '大阪倉庫',
    quantity_available: 0,
    quantity_reserved: 0,
    reorder_point: 20,
    status: 'out_of_stock',
    version: 1,
    created_at: '2024-01-15T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // 在庫一覧取得
  http.get('/bff/api/v1/list_inventory', () => {
    return HttpResponse.json({ items: mockInventoryItems });
  }),

  // 在庫個別取得
  http.get('/bff/api/v1/get_inventory/:id', ({ params }) => {
    const item = mockInventoryItems.find((i) => i.id === params.id);
    if (!item) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(item);
  }),

  // 在庫予約
  http.post('/bff/api/v1/reserve_stock', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const item = mockInventoryItems.find((i) => i.product_id === body.product_id);
    if (!item) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({
      ...item,
      quantity_available: item.quantity_available - (body.quantity as number),
      quantity_reserved: item.quantity_reserved + (body.quantity as number),
      version: item.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 在庫予約解放
  http.post('/bff/api/v1/release_stock', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const item = mockInventoryItems.find((i) => i.product_id === body.product_id);
    if (!item) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({
      ...item,
      quantity_available: item.quantity_available + (body.quantity as number),
      quantity_reserved: item.quantity_reserved - (body.quantity as number),
      version: item.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 在庫更新
  http.put('/bff/api/v1/update_stock/:id', async ({ params, request }) => {
    const item = mockInventoryItems.find((i) => i.id === params.id);
    if (!item) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...item,
      ...body,
      version: item.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockInventoryItems };
