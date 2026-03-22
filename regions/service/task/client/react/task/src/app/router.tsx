import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const TaskList = lazy(() => import('../features/tasks/TaskList').then((m) => ({ default: m.TaskList })));
const TaskDetail = lazy(() => import('../features/tasks/TaskDetail').then((m) => ({ default: m.TaskDetail })));
const TaskForm = lazy(() => import('../features/tasks/TaskForm').then((m) => ({ default: m.TaskForm })));

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav aria-label="メインナビゲーション" style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/" style={{ marginRight: '16px' }}>
          タスク一覧
        </Link>
        <Link to="/tasks/new">新規タスク</Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: ルートパスでタスク一覧を表示
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: TaskList,
});

// タスク新規作成ルート
const taskNewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks/new',
  component: TaskForm,
});

// タスク詳細ルート
const taskDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks/$id',
  component: () => {
    const { id } = taskDetailRoute.useParams();
    return <TaskDetail taskId={id} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([indexRoute, taskNewRoute, taskDetailRoute]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
