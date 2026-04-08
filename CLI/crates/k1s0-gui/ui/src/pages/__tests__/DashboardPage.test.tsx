/// DashboardPage.test.tsx: DashboardPage のユニットテスト。
/// LOW-009 監査対応: DashboardPage のテストカバレッジを追加する。
/// クイックアクションリンクとメトリクス表示を検証する。
/// DashboardPage は @tanstack/react-router の Link を使用するため RouterProvider でラップする。

import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { render } from '@testing-library/react';
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import DashboardPage from '../DashboardPage';

// DashboardPage は Link コンポーネントを使用するため RouterProvider でラップする
function renderDashboard() {
  const rootRoute = createRootRoute({
    component: DashboardPage,
  });

  // DashboardPage の Link が参照するルートを登録する
  const routes = [
    '/init', '/generate', '/deps', '/dev', '/migrate',
    '/template-migrate', '/config-types', '/navigation-types',
    '/event-codegen', '/validate', '/build', '/test', '/deploy',
  ].map((path) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path,
      component: () => null,
    }),
  );

  const history = createMemoryHistory({ initialEntries: ['/'] });
  const router = createRouter({ routeTree: rootRoute.addChildren(routes), history });

  return render(<RouterProvider router={router} />);
}

describe('DashboardPage', () => {
  it('ページが正常にレンダリングされる', async () => {
    renderDashboard();
    expect(await screen.findByTestId('dashboard-page')).toBeInTheDocument();
  });

  it('すべてのクイックアクションリンクが表示される', async () => {
    renderDashboard();
    // まずページが表示されるまで待つ
    await screen.findByTestId('dashboard-page');
    // ダッシュボードに表示されるすべての主要アクションリンクを確認する
    const expectedActions = [
      'init',
      'generate',
      'deps',
      'dev',
      'migrate',
      'template-migrate',
      'config-types',
      'navigation-types',
      'event-codegen',
      'validate',
      'build',
      'test',
      'deploy',
    ];
    for (const action of expectedActions) {
      expect(screen.getByTestId(`dashboard-link-${action}`)).toBeInTheDocument();
    }
  });

  it('メトリクスカードが表示される', async () => {
    renderDashboard();
    // メトリクスカードのラベルが存在することを確認する
    expect(await screen.findByText('プライマリルート')).toBeInTheDocument();
    expect(screen.getByText('初期化ルート')).toBeInTheDocument();
    expect(screen.getByText('クイックアクション')).toBeInTheDocument();
  });

  it('クイックアクション数が正しい', async () => {
    renderDashboard();
    // 13件のクイックアクションが存在することを確認する
    await screen.findByTestId('dashboard-page');
    expect(screen.getByText('13')).toBeInTheDocument();
  });
});
