import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';
import { OrderList } from '../features/orders/OrderList';
import { OrderDetail } from '../features/orders/OrderDetail';
import { OrderForm } from '../features/orders/OrderForm';

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/" style={{ marginRight: '16px' }}>
          注文一覧
        </Link>
        <Link to="/orders/new">新規注文</Link>
      </nav>
      {/* 子ルートの描画領域 */}
      <Outlet />
    </div>
  ),
});

// インデックスルート: ルートパスで注文一覧を表示
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: OrderList,
});

// 注文新規作成ルート
const orderNewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/orders/new',
  component: OrderForm,
});

// 注文詳細ルート
const orderDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/orders/$id',
  component: () => {
    const { id } = orderDetailRoute.useParams();
    return <OrderDetail orderId={id} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([indexRoute, orderNewRoute, orderDetailRoute]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
