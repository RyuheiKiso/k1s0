import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import type {
  ProjectType,
  StatusDefinition,
  StatusDefinitionVersion,
  TenantProjectExtension,
} from '../../src/types/projectMaster';

// テスト用モックデータ: プロジェクトタイプ
const mockProjectTypes: ProjectType[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    code: 'SOFTWARE',
    display_name: 'ソフトウェア開発',
    description: 'ソフトウェアプロジェクトの管理',
    default_workflow: null,
    is_active: true,
    sort_order: 1,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    code: 'INFRA',
    display_name: 'インフラ構築',
    description: 'インフラプロジェクトの管理',
    default_workflow: null,
    is_active: true,
    sort_order: 2,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: ステータス定義
const mockStatusDefinitions: StatusDefinition[] = [
  {
    id: '660e8400-e29b-41d4-a716-446655440001',
    project_type_id: '550e8400-e29b-41d4-a716-446655440001',
    code: 'TODO',
    display_name: '未着手',
    description: 'まだ開始していないタスク',
    color: '#95a5a6',
    allowed_transitions: ['IN_PROGRESS'],
    is_initial: true,
    is_terminal: false,
    sort_order: 1,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '660e8400-e29b-41d4-a716-446655440002',
    project_type_id: '550e8400-e29b-41d4-a716-446655440001',
    code: 'DONE',
    display_name: '完了',
    description: '完了したタスク',
    color: '#27ae60',
    allowed_transitions: null,
    is_initial: false,
    is_terminal: true,
    sort_order: 3,
    created_by: 'admin',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: バージョン履歴
const mockVersions: StatusDefinitionVersion[] = [
  {
    id: '770e8400-e29b-41d4-a716-446655440001',
    status_definition_id: '660e8400-e29b-41d4-a716-446655440001',
    version_number: 1,
    before_data: null,
    after_data: { display_name: '未着手', code: 'TODO' },
    changed_by: 'admin',
    change_reason: '初期作成',
    created_at: '2024-01-01T00:00:00Z',
  },
];

// テスト用モックデータ: テナント拡張
const mockTenantExtension: TenantProjectExtension = {
  id: '880e8400-e29b-41d4-a716-446655440001',
  tenant_id: 'tenant-001',
  status_definition_id: '660e8400-e29b-41d4-a716-446655440001',
  display_name_override: 'バックログ',
  attributes_override: null,
  is_enabled: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

// MSWハンドラー定義: 全APIエンドポイントのモック
const handlers = [
  // プロジェクトタイプ一覧取得
  http.get('/bff/api/v1/taskmanagement/project-types', () => {
    return HttpResponse.json({ project_types: mockProjectTypes });
  }),

  // プロジェクトタイプ作成
  http.post('/bff/api/v1/taskmanagement/project-types', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newProjectType: ProjectType = {
      id: crypto.randomUUID(),
      code: body.code as string,
      display_name: body.display_name as string,
      description: (body.description as string) ?? null,
      default_workflow: (body.default_workflow as string) ?? null,
      is_active: (body.is_active as boolean) ?? true,
      sort_order: (body.sort_order as number) ?? 0,
      created_by: 'test-user',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    return HttpResponse.json(newProjectType, { status: 201 });
  }),

  // プロジェクトタイプ個別取得
  http.get('/bff/api/v1/taskmanagement/project-types/:id', ({ params }) => {
    const projectType = mockProjectTypes.find((p) => p.id === params.id);
    if (!projectType) return new HttpResponse(null, { status: 404 });
    return HttpResponse.json(projectType);
  }),

  // プロジェクトタイプ更新
  http.put('/bff/api/v1/taskmanagement/project-types/:id', async ({ params, request }) => {
    const projectType = mockProjectTypes.find((p) => p.id === params.id);
    if (!projectType) return new HttpResponse(null, { status: 404 });
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...projectType,
      ...body,
      updated_at: new Date().toISOString(),
    });
  }),

  // プロジェクトタイプ削除
  http.delete('/bff/api/v1/taskmanagement/project-types/:id', () => {
    return new HttpResponse(null, { status: 204 });
  }),

  // ステータス定義一覧取得
  http.get('/bff/api/v1/taskmanagement/project-types/:id/status-definitions', () => {
    return HttpResponse.json({ status_definitions: mockStatusDefinitions });
  }),

  // ステータス定義作成
  http.post(
    '/bff/api/v1/taskmanagement/project-types/:id/status-definitions',
    async ({ params, request }) => {
      const body = (await request.json()) as Record<string, unknown>;
      const newDef: StatusDefinition = {
        id: crypto.randomUUID(),
        project_type_id: params.id as string,
        code: body.code as string,
        display_name: body.display_name as string,
        description: (body.description as string) ?? null,
        color: (body.color as string) ?? null,
        allowed_transitions: (body.allowed_transitions as string[]) ?? null,
        is_initial: (body.is_initial as boolean) ?? false,
        is_terminal: (body.is_terminal as boolean) ?? false,
        sort_order: (body.sort_order as number) ?? 0,
        created_by: 'test-user',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      return HttpResponse.json(newDef, { status: 201 });
    }
  ),

  // ステータス定義バージョン履歴取得
  http.get(
    '/bff/api/v1/taskmanagement/status-definitions/:statusDefinitionId/versions',
    () => {
      return HttpResponse.json({ versions: mockVersions });
    }
  ),

  // テナント拡張一覧取得
  http.get('/bff/api/v1/taskmanagement/tenant-extensions', () => {
    return HttpResponse.json({ extensions: [mockTenantExtension] });
  }),

  // テナント拡張更新（upsert）
  http.put('/bff/api/v1/taskmanagement/tenant-extensions', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      ...mockTenantExtension,
      ...body,
      updated_at: new Date().toISOString(),
    });
  }),

  // テナント拡張削除
  http.delete('/bff/api/v1/taskmanagement/tenant-extensions', () => {
    return new HttpResponse(null, { status: 204 });
  }),
];

// MSWサーバーインスタンスのエクスポート
export const server = setupServer(...handlers);

// テスト用モックデータのエクスポート
export { mockProjectTypes, mockStatusDefinitions, mockVersions, mockTenantExtension };
