import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';
import { InventoryList } from '../features/inventory/InventoryList';
import { InventoryDetail } from '../features/inventory/InventoryDetail';

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/" style={{ marginRight: '16px' }}>
          在庫管理
        </Link>
      </nav>
      {/* 子ルートの描画領域 */}
      <Outlet />
    </div>
  ),
});

// インデックスルート: ルートパスで在庫一覧を表示
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: InventoryList,
});

// 在庫詳細ルート: 個別の在庫アイテムと在庫操作を表示
const inventoryDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/inventory/$id',
  component: () => {
    const { id } = inventoryDetailRoute.useParams();
    return <InventoryDetail id={id} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([indexRoute, inventoryDetailRoute]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
