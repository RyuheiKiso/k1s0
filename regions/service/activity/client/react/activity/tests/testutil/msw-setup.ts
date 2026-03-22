import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { Activity } from '../../src/types/activity';

// テスト用モックデータ: アクティビティ一覧
const mockActivities: Activity[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    task_id: 'TASK-001',
    actor_id: 'USER-001',
    activity_type: 'comment',
    content: 'テストコメントです',
    duration_minutes: null,
    status: 'active',
    metadata: null,
    idempotency_key: null,
    version: 1,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    task_id: 'TASK-001',
    actor_id: 'USER-002',
    activity_type: 'time_entry',
    content: null,
    duration_minutes: 60,
    status: 'submitted',
    metadata: null,
    idempotency_key: 'KEY-001',
    version: 2,
    created_at: '2024-01-16T14:30:00Z',
    updated_at: '2024-01-16T15:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // アクティビティ一覧取得
  http.get('/bff/api/v1/activities', () => {
    return HttpResponse.json({ activities: mockActivities });
  }),

  // アクティビティ個別取得
  http.get('/bff/api/v1/activities/:id', ({ params }) => {
    const activity = mockActivities.find((a) => a.id === params.id);
    if (!activity) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(activity);
  }),

  // アクティビティ作成
  http.post('/bff/api/v1/activities', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newActivity: Activity = {
      id: crypto.randomUUID(),
      task_id: body.task_id as string,
      actor_id: body.actor_id as string,
      activity_type: body.activity_type as Activity['activity_type'],
      content: (body.content as string) ?? null,
      duration_minutes: (body.duration_minutes as number) ?? null,
      status: 'active',
      metadata: null,
      idempotency_key: (body.idempotency_key as string) ?? null,
      version: 1,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newActivity, { status: 201 });
  }),

  // 承認申請
  http.post('/bff/api/v1/activities/:id/submit', ({ params }) => {
    const activity = mockActivities.find((a) => a.id === params.id);
    if (!activity) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({
      ...activity,
      status: 'submitted',
      version: activity.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 承認
  http.post('/bff/api/v1/activities/:id/approve', ({ params }) => {
    const activity = mockActivities.find((a) => a.id === params.id);
    if (!activity) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({
      ...activity,
      status: 'approved',
      version: activity.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),

  // 却下
  http.post('/bff/api/v1/activities/:id/reject', ({ params }) => {
    const activity = mockActivities.find((a) => a.id === params.id);
    if (!activity) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({
      ...activity,
      status: 'rejected',
      version: activity.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockActivities };
