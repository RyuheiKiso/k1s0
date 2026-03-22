import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';

// ルートコンポーネントの遅延読み込み（コード分割）
const ProjectTypeList = lazy(() =>
  import('../features/project-types/ProjectTypeList').then((m) => ({
    default: m.ProjectTypeList,
  }))
);
const ProjectTypeDetail = lazy(() =>
  import('../features/project-types/ProjectTypeDetail').then((m) => ({
    default: m.ProjectTypeDetail,
  }))
);
const StatusDefinitionDetail = lazy(() =>
  import('../features/status-definitions/StatusDefinitionDetail').then((m) => ({
    default: m.StatusDefinitionDetail,
  }))
);
const TenantExtensionList = lazy(() =>
  import('../features/tenant-extensions/TenantExtensionList').then((m) => ({
    default: m.TenantExtensionList,
  }))
);
const TenantExtensionForm = lazy(() =>
  import('../features/tenant-extensions/TenantExtensionForm').then((m) => ({
    default: m.TenantExtensionForm,
  }))
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
        <Link to="/project-types" style={{ marginRight: '16px' }}>
          プロジェクトタイプ管理
        </Link>
      </nav>
      {/* 子ルートの描画領域（Suspenseでローディング表示） */}
      <Suspense fallback={<div>読み込み中...</div>}>
        <Outlet />
      </Suspense>
    </div>
  ),
});

// インデックスルート: ルートパスからプロジェクトタイプ一覧へリダイレクト
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => {
    // ルートアクセス時はプロジェクトタイプ一覧を表示
    return <ProjectTypeList />;
  },
});

// プロジェクトタイプ一覧ルート
const projectTypesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/project-types',
  component: ProjectTypeList,
});

// プロジェクトタイプ詳細（ステータス定義一覧を含む）ルート
const projectTypeDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/project-types/$projectTypeId/status-definitions',
  component: () => {
    const { projectTypeId } = projectTypeDetailRoute.useParams();
    return <ProjectTypeDetail projectTypeId={projectTypeId} />;
  },
});

// ステータス定義のバージョン履歴ルート
const statusDefinitionVersionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/status-definitions/$statusDefinitionId/versions',
  component: () => {
    const { statusDefinitionId } = statusDefinitionVersionsRoute.useParams();
    return <StatusDefinitionDetail statusDefinitionId={statusDefinitionId} />;
  },
});

// テナント拡張一覧ルート
const tenantExtensionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenants/$tenantId/extensions',
  component: () => {
    const { tenantId } = tenantExtensionsRoute.useParams();
    return <TenantExtensionList tenantId={tenantId} />;
  },
});

// テナント拡張編集ルート
const tenantExtensionEditRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenants/$tenantId/extensions/$statusDefinitionId',
  component: () => {
    const { tenantId, statusDefinitionId } = tenantExtensionEditRoute.useParams();
    return <TenantExtensionForm tenantId={tenantId} statusDefinitionId={statusDefinitionId} />;
  },
});

// ルートツリーの構築: 全ルートを登録
const routeTree = rootRoute.addChildren([
  indexRoute,
  projectTypesRoute,
  projectTypeDetailRoute,
  statusDefinitionVersionsRoute,
  tenantExtensionsRoute,
  tenantExtensionEditRoute,
]);

// ルーターインスタンスの作成
export const router = createRouter({ routeTree });

// TanStack Router の型安全性のための型宣言
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
