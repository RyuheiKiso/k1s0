import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const ActivityList = lazy(() =>
  import('../features/activities/ActivityList').then((m) => ({ default: m.ActivityList }))
);
const ActivityDetail = lazy(() =>
  import('../features/activities/ActivityDetail').then((m) => ({ default: m.ActivityDetail }))
);
const ActivityForm = lazy(() =>
  import('../features/activities/ActivityForm').then((m) => ({ default: m.ActivityForm }))
);

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav
        aria-label="メインナビゲーション"
        style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}
      >
        <Link to="/" style={{ marginRight: '16px' }}>
          アクティビティ一覧
        </Link>
        <Link to="/activities/new">新規アクティビティ</Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: ルートパスでアクティビティ一覧を表示
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: ActivityList,
});

// アクティビティ新規作成ルート
const activityNewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/activities/new',
  component: ActivityForm,
});

// アクティビティ詳細ルート
const activityDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/activities/$id',
  component: () => {
    const { id } = activityDetailRoute.useParams();
    return <ActivityDetail activityId={id} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([indexRoute, activityNewRoute, activityDetailRoute]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
