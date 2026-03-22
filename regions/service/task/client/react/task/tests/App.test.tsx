import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { QueryClientProvider } from '@tanstack/react-query';
import { QueryClient } from '@tanstack/react-query';
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
} from '@tanstack/react-router';
import { server } from './testutil/msw-setup';
import { TaskList } from '../src/features/tasks/TaskList';

// MSWサーバーのライフサイクル管理
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// テスト用のQueryClientラッパー: 各テストで独立したキャッシュを使用
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
}

// テスト用ラッパーコンポーネント: TanStack RouterのRouterProviderでラップ
function TestWrapper({ children }: { children: React.ReactNode }) {
  const queryClient = createTestQueryClient();

  // テスト用ルートルート: childrenをそのままレンダリング
  const rootRoute = createRootRoute({
    component: () => <>{children}</>,
  });

  // インデックスルート: ルートパスに対応
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <>{children}</>,
  });

  // タスク詳細ルート: useNavigateのナビゲーション先として必要
  const detailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/tasks/$id',
    component: () => <div>detail</div>,
  });

  // テスト用ルーターの構築
  const routeTree = rootRoute.addChildren([indexRoute, detailRoute]);
  const memoryHistory = createMemoryHistory({ initialEntries: ['/'] });
  const router = createRouter({ routeTree, history: memoryHistory });

  return (
    <QueryClientProvider client={queryClient}>
      {/* @ts-expect-error テスト用ルーターの型はアプリルーターと異なる */}
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

describe('TaskList', () => {
  // タスク一覧が正常にレンダリングされることを確認
  it('タスク一覧を表示する', async () => {
    render(
      <TestWrapper>
        <TaskList />
      </TestWrapper>
    );

    // ローディング表示の確認（RouterProvider経由のため非同期で検索）
    expect(await screen.findByText('読み込み中...')).toBeInTheDocument();

    // データ取得後にタスクが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('テストタスクA')).toBeInTheDocument();
      expect(screen.getByText('テストタスクB')).toBeInTheDocument();
    });
  });

  // ページタイトルが表示されることを確認
  it('ページタイトルを表示する', async () => {
    render(
      <TestWrapper>
        <TaskList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('タスク一覧')).toBeInTheDocument();
    });
  });

  // ステータスフィルターが存在することを確認
  it('ステータスフィルターを表示する', async () => {
    render(
      <TestWrapper>
        <TaskList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByLabelText('ステータス:')).toBeInTheDocument();
    });
  });
});
