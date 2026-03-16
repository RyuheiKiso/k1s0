import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { Payment } from '../../src/types/payment';

// テスト用モックデータ: 決済一覧（サーバー契約に準拠）
const mockPayments: Payment[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    order_id: 'ORD-001',
    customer_id: 'CUS-001',
    amount: 15000,
    currency: 'JPY',
    // サーバー契約に準拠: pending→initiated
    status: 'initiated',
    payment_method: 'credit_card',
    transaction_id: null,
    error_code: null,
    error_message: null,
    version: 1,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    order_id: 'ORD-002',
    customer_id: 'CUS-002',
    amount: 32000,
    currency: 'JPY',
    status: 'completed',
    payment_method: 'bank_transfer',
    transaction_id: 'TXN-ABC-123',
    error_code: null,
    error_message: null,
    version: 2,
    created_at: '2024-01-16T14:30:00Z',
    updated_at: '2024-01-16T15:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440003',
    order_id: 'ORD-003',
    customer_id: 'CUS-001',
    amount: 5500,
    currency: 'JPY',
    status: 'failed',
    payment_method: 'convenience_store',
    transaction_id: null,
    // サーバー契約に準拠: failure_reason→error_code/error_message
    error_code: 'PAYMENT_EXPIRED',
    error_message: '支払い期限切れ',
    version: 2,
    created_at: '2024-01-17T09:00:00Z',
    updated_at: '2024-01-18T00:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック（サーバー契約に準拠）
const handlers = [
  // 決済一覧取得
  http.get('/bff/api/v1/list_payments', () => {
    return HttpResponse.json({ payments: mockPayments });
  }),

  // 決済個別取得
  http.get('/bff/api/v1/get_payment/:id', ({ params }) => {
    const payment = mockPayments.find((p) => p.id === params.id);
    if (!payment) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(payment);
  }),

  // 決済開始（サーバー契約に準拠: status=initiated, error_code/error_message, version付き）
  http.post('/bff/api/v1/initiate_payment', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newPayment: Payment = {
      id: crypto.randomUUID(),
      order_id: body.order_id as string,
      customer_id: body.customer_id as string,
      amount: body.amount as number,
      currency: (body.currency as string) ?? 'JPY',
      status: 'initiated',
      payment_method: (body.payment_method as string) ?? null,
      transaction_id: null,
      error_code: null,
      error_message: null,
      version: 1,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newPayment, { status: 201 });
  }),

  // 決済完了（リクエストボディからtransaction_idを取得）
  http.post('/bff/api/v1/complete_payment/:id', async ({ params, request }) => {
    const payment = mockPayments.find((p) => p.id === params.id);
    if (!payment) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...payment,
      status: 'completed',
      transaction_id: (body.transaction_id as string) ?? `TXN-${Date.now()}`,
      version: payment.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 決済失敗（リクエストボディからerror_code/error_messageを取得）
  http.post('/bff/api/v1/fail_payment/:id', async ({ params, request }) => {
    const payment = mockPayments.find((p) => p.id === params.id);
    if (!payment) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...payment,
      status: 'failed',
      error_code: (body.error_code as string) ?? 'UNKNOWN',
      error_message: (body.error_message as string) ?? '処理エラー',
      version: payment.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 決済返金（リクエストbodyからreasonを受け取る）
  http.post('/bff/api/v1/refund_payment/:id', async ({ params, request }) => {
    const payment = mockPayments.find((p) => p.id === params.id);
    if (!payment) return new HttpResponse(null, { status: 404 });
    // hooksがbodyで送信するreasonを読み取る
    await request.json();
    return HttpResponse.json({
      ...payment,
      status: 'refunded',
      version: payment.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockPayments };
