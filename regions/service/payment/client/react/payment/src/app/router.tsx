import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const PaymentList = lazy(() => import('../features/payments/PaymentList').then((m) => ({ default: m.PaymentList })));
const PaymentDetail = lazy(() => import('../features/payments/PaymentDetail').then((m) => ({ default: m.PaymentDetail })));
const PaymentForm = lazy(() => import('../features/payments/PaymentForm').then((m) => ({ default: m.PaymentForm })));

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav aria-label="メインナビゲーション" style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/" style={{ marginRight: '16px' }}>
          決済一覧
        </Link>
        <Link to="/payments/new" style={{ marginRight: '16px' }}>
          新規決済
        </Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: 決済一覧を表示
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: PaymentList,
});

// 決済新規作成ルート
const paymentNewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/payments/new',
  component: PaymentForm,
});

// 決済詳細ルート
const paymentDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/payments/$id',
  component: () => {
    const { id } = paymentDetailRoute.useParams();
    return <PaymentDetail id={id} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([
  indexRoute,
  paymentNewRoute,
  paymentDetailRoute,
]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
