import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { Order } from '../../src/types/order';

// テスト用モックデータ: 注文一覧
const mockOrders: Order[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    customer_id: 'CUST-001',
    status: 'pending',
    total_amount: 15000,
    currency: 'JPY',
    items: [
      {
        product_id: 'PROD-001',
        product_name: 'テスト商品A',
        quantity: 2,
        unit_price: 5000,
        subtotal: 10000,
      },
      {
        product_id: 'PROD-002',
        product_name: 'テスト商品B',
        quantity: 1,
        unit_price: 5000,
        subtotal: 5000,
      },
    ],
    notes: 'テスト注文です',
    version: 1,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    customer_id: 'CUST-002',
    status: 'confirmed',
    total_amount: 8000,
    currency: 'JPY',
    items: [
      {
        product_id: 'PROD-003',
        product_name: 'テスト商品C',
        quantity: 4,
        unit_price: 2000,
        subtotal: 8000,
      },
    ],
    notes: null,
    version: 2,
    created_at: '2024-01-16T14:30:00Z',
    updated_at: '2024-01-16T15:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // 注文一覧取得
  http.get('/bff/api/v1/list_orders', () => {
    return HttpResponse.json({ orders: mockOrders });
  }),

  // 注文個別取得
  http.get('/bff/api/v1/get_order/:id', ({ params }) => {
    const order = mockOrders.find((o) => o.id === params.id);
    if (!order) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(order);
  }),

  // 注文作成
  http.post('/bff/api/v1/create_order', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const items = body.items as Array<{
      product_id: string;
      product_name: string;
      quantity: number;
      unit_price: number;
    }>;
    const orderItems = items.map((item) => ({
      ...item,
      subtotal: item.quantity * item.unit_price,
    }));
    const totalAmount = orderItems.reduce((sum, item) => sum + item.subtotal, 0);

    const newOrder: Order = {
      id: crypto.randomUUID(),
      customer_id: body.customer_id as string,
      status: 'pending',
      total_amount: totalAmount,
      currency: (body.currency as string) ?? 'JPY',
      items: orderItems,
      notes: (body.notes as string) ?? null,
      version: 1,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newOrder, { status: 201 });
  }),

  // 注文ステータス更新
  http.put('/bff/api/v1/update_order_status/:id', async ({ params, request }) => {
    const order = mockOrders.find((o) => o.id === params.id);
    if (!order) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...order,
      status: body.status,
      version: order.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockOrders };
