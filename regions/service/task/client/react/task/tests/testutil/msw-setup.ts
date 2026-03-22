import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type { Task } from '../../src/types/task';

// テスト用モックデータ: タスク一覧
const mockTasks: Task[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    project_id: 'PROJ-001',
    title: 'テストタスクA',
    description: 'テスト用のタスクです',
    status: 'open',
    priority: 'high',
    assignee_id: 'USER-001',
    reporter_id: 'USER-002',
    due_date: '2026-04-01',
    labels: ['bug', 'frontend'],
    created_by: 'USER-002',
    updated_by: 'USER-002',
    version: 1,
    created_at: '2026-01-15T10:00:00Z',
    updated_at: '2026-01-15T10:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    project_id: 'PROJ-001',
    title: 'テストタスクB',
    description: null,
    status: 'in_progress',
    priority: 'medium',
    assignee_id: null,
    reporter_id: 'USER-001',
    due_date: null,
    labels: [],
    created_by: 'USER-001',
    updated_by: 'USER-001',
    version: 2,
    created_at: '2026-01-16T14:30:00Z',
    updated_at: '2026-01-16T15:00:00Z',
  },
];

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // タスク一覧取得
  http.get('/bff/api/v1/tasks', () => {
    return HttpResponse.json({ tasks: mockTasks });
  }),

  // タスク個別取得
  http.get('/bff/api/v1/tasks/:id', ({ params }) => {
    const task = mockTasks.find((t) => t.id === params.id);
    if (!task) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(task);
  }),

  // タスク作成
  http.post('/bff/api/v1/tasks', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;

    const newTask: Task = {
      id: crypto.randomUUID(),
      project_id: body.project_id as string,
      title: body.title as string,
      description: (body.description as string) ?? null,
      status: 'open',
      priority: (body.priority as Task['priority']) ?? 'medium',
      assignee_id: (body.assignee_id as string) ?? null,
      reporter_id: 'USER-001',
      due_date: (body.due_date as string) ?? null,
      labels: (body.labels as string[]) ?? [],
      created_by: 'USER-001',
      updated_by: 'USER-001',
      version: 1,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newTask, { status: 201 });
  }),

  // タスクステータス更新
  http.put('/bff/api/v1/tasks/:id/status', async ({ params, request }) => {
    const task = mockTasks.find((t) => t.id === params.id);
    if (!task) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...task,
      status: body.status,
      version: task.version + 1,
      updated_at: new Date().toISOString(),
    });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockTasks };
