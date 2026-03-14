import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';
import { PaymentList } from '../features/payments/PaymentList';
import { PaymentDetail } from '../features/payments/PaymentDetail';
import { PaymentForm } from '../features/payments/PaymentForm';

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/" style={{ marginRight: '16px' }}>
          決済一覧
        </Link>
        <Link to="/payments/new" style={{ marginRight: '16px' }}>
          新規決済
        </Link>
      </nav>
      {/* 子ルートの描画領域 */}
      <Outlet />
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
