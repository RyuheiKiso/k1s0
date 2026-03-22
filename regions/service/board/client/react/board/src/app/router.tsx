import { lazy, Suspense, useState } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const BoardView = lazy(() => import('../features/board/BoardView').then((m) => ({ default: m.BoardView })));

// プロジェクトID選択コンポーネント: URLパラメータがない場合の入力フォーム
function ProjectSelector() {
  const [projectId, setProjectId] = useState('');

  return (
    <div style={{ padding: '16px' }}>
      <h1>ボードを表示する</h1>
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label htmlFor="project-id">プロジェクトID:</label>
        <input
          id="project-id"
          value={projectId}
          onChange={(e) => setProjectId(e.target.value)}
          placeholder="プロジェクトIDを入力"
          aria-label="プロジェクトID"
        />
        <Link
          to="/boards/$projectId"
          params={{ projectId }}
          aria-label="ボードを表示"
        >
          表示
        </Link>
      </div>
    </div>
  );
}

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1400px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav aria-label="メインナビゲーション" style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/">
          ボードホーム
        </Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: プロジェクトID入力画面
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: ProjectSelector,
});

// ボード表示ルート: プロジェクトIDを受け取りKanbanボードを表示
const boardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/boards/$projectId',
  component: () => {
    const { projectId } = boardRoute.useParams();
    return <BoardView projectId={projectId} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([indexRoute, boardRoute]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
