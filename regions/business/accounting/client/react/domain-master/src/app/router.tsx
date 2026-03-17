import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const CategoryList = lazy(() => import('../features/categories/CategoryList').then((m) => ({ default: m.CategoryList })));
const ItemList = lazy(() => import('../features/items/ItemList').then((m) => ({ default: m.ItemList })));
const VersionHistory = lazy(() => import('../features/versions/VersionHistory').then((m) => ({ default: m.VersionHistory })));
const TenantExtensionForm = lazy(() => import('../features/tenant-extensions/TenantExtensionForm').then((m) => ({ default: m.TenantExtensionForm })));

// ルートレイアウト: 全ページ共通のナビゲーションヘッダー
const rootRoute = createRootRoute({
  component: () => (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '16px' }}>
      {/* グローバルナビゲーション */}
      <nav aria-label="メインナビゲーション" style={{ borderBottom: '1px solid #ccc', paddingBottom: '8px', marginBottom: '16px' }}>
        <Link to="/categories" style={{ marginRight: '16px' }}>
          カテゴリ管理
        </Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: ルートパスからカテゴリ一覧へリダイレクト
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => {
    // ルートアクセス時はカテゴリ一覧を表示
    return <CategoryList />;
  },
});

// カテゴリ一覧ルート
const categoriesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/categories',
  component: CategoryList,
});

// カテゴリ配下のアイテム一覧ルート
const categoryItemsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/categories/$code/items',
  component: () => {
    const { code } = categoryItemsRoute.useParams();
    return <ItemList categoryCode={code} />;
  },
});

// アイテムのバージョン履歴ルート
const itemVersionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/categories/$code/items/$itemCode/versions',
  component: () => {
    const { code, itemCode } = itemVersionsRoute.useParams();
    return <VersionHistory categoryCode={code} itemCode={itemCode} />;
  },
});

// テナント拡張管理ルート
const tenantExtensionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenants/$tenantId/items/$itemId',
  component: () => {
    const { tenantId, itemId } = tenantExtensionRoute.useParams();
    return <TenantExtensionForm tenantId={tenantId} itemId={itemId} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([
  indexRoute,
  categoriesRoute,
  categoryItemsRoute,
  itemVersionsRoute,
  tenantExtensionRoute,
]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
