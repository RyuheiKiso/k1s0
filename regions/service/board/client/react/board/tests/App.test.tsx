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
import { BoardView } from '../src/features/board/BoardView';

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

  // テスト用ルーターの構築
  const routeTree = rootRoute.addChildren([indexRoute]);
  const memoryHistory = createMemoryHistory({ initialEntries: ['/'] });
  const router = createRouter({ routeTree, history: memoryHistory });

  return (
    <QueryClientProvider client={queryClient}>
      {/* @ts-expect-error テスト用ルーターの型はアプリルーターと異なる */}
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

describe('BoardView', () => {
  // ボード画面が正常にレンダリングされることを確認
  it('ボードタイトルを表示する', async () => {
    render(
      <TestWrapper>
        <BoardView projectId="PROJECT-001" />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Kanbanボード')).toBeInTheDocument();
    });
  });

  // カラム一覧が表示されることを確認
  it('カラム一覧を表示する', async () => {
    render(
      <TestWrapper>
        <BoardView projectId="PROJECT-001" />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('todo')).toBeInTheDocument();
      expect(screen.getByText('in_progress')).toBeInTheDocument();
      expect(screen.getByText('done')).toBeInTheDocument();
    });
  });

  // プロジェクトIDが表示されることを確認
  it('プロジェクトIDを表示する', async () => {
    render(
      <TestWrapper>
        <BoardView projectId="PROJECT-001" />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText(/PROJECT-001/)).toBeInTheDocument();
    });
  });
});
