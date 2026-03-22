import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { BoardColumn } from '../../src/types/board';

// テスト用モックデータ: ボードカラム一覧
const mockColumns: BoardColumn[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    project_id: 'PROJECT-001',
    status_code: 'todo',
    wip_limit: 5,
    task_count: 3,
    version: 1,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    project_id: 'PROJECT-001',
    status_code: 'in_progress',
    wip_limit: 3,
    task_count: 2,
    version: 1,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440003',
    project_id: 'PROJECT-001',
    status_code: 'done',
    wip_limit: 0,
    task_count: 5,
    version: 2,
    created_at: '2024-01-15T10:00:00Z',
    updated_at: '2024-01-16T09:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // カラム一覧取得
  http.get('/bff/api/v1/boards/:projectId/columns', ({ params }) => {
    const cols = mockColumns.filter((c) => c.project_id === params.projectId);
    return HttpResponse.json({ columns: cols });
  }),

  // 単一カラム取得
  http.get('/bff/api/v1/boards/:projectId/columns/:statusCode', ({ params }) => {
    const col = mockColumns.find(
      (c) => c.project_id === params.projectId && c.status_code === params.statusCode
    );
    if (!col) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(col);
  }),

  // タスク数インクリメント
  http.post('/bff/api/v1/boards/increment', async ({ request }) => {
    const body = (await request.json()) as Record<string, string>;
    const col = mockColumns.find(
      (c) => c.project_id === body.project_id && c.status_code === body.status_code
    );
    if (!col) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json({ ...col, task_count: col.task_count + 1, version: col.version + 1 });
  }),

  // タスク数デクリメント
  http.post('/bff/api/v1/boards/decrement', async ({ request }) => {
    const body = (await request.json()) as Record<string, string>;
    const col = mockColumns.find(
      (c) => c.project_id === body.project_id && c.status_code === body.status_code
    );
    if (!col) return new HttpResponse(null, { status: 404 });
    const newCount = Math.max(0, col.task_count - 1);
    return HttpResponse.json({ ...col, task_count: newCount, version: col.version + 1 });
  }),

  // WIP制限更新
  http.put('/bff/api/v1/boards/:projectId/columns/:statusCode/wip-limit', async ({ params, request }) => {
    const col = mockColumns.find(
      (c) => c.project_id === params.projectId && c.status_code === params.statusCode
    );
    if (!col) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, number>;
    return HttpResponse.json({
      ...col,
      wip_limit: body.wip_limit,
      version: col.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockColumns };
