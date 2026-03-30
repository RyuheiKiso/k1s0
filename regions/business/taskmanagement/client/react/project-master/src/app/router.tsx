import { lazy, Suspense } from 'react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  Link,
} from '@tanstack/react-router';
// H-12 監査対応: 未認証ユーザーが管理画面へアクセスするのを防ぐため認証ガードを使用する
// system-client パッケージの ProtectedRoute はロールベースアクセス制御（RBAC）に対応している
import { ProtectedRoute } from '@k1s0/system-client';

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

// H-12 監査対応: 未認証ユーザー向けフォールバック表示コンポーネント
// ログインページへのリンクを提供し、ユーザーが認証を完了できるよう案内する
function AuthRequired() {
  return (
    <div role="alert" style={{ textAlign: 'center', padding: '32px' }}>
      <p>このページを表示するにはログインが必要です。</p>
      <a href="/bff/auth/login">ログインページへ</a>
    </div>
  );
}

// インデックスルート: ルートパスからプロジェクトタイプ一覧へリダイレクト
// H-12 監査対応: 全ルートを ProtectedRoute でラップし、未認証アクセスをブロックする
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => {
    // ルートアクセス時はプロジェクトタイプ一覧を表示（認証済みユーザーのみ）
    return (
      <ProtectedRoute fallback={<AuthRequired />}>
        <ProjectTypeList />
      </ProtectedRoute>
    );
  },
});

// プロジェクトタイプ一覧ルート（認証ガード付き）
const projectTypesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/project-types',
  component: () => (
    // H-12 監査対応: プロジェクトタイプ管理は認証済みユーザーのみアクセス可能
    <ProtectedRoute fallback={<AuthRequired />}>
      <ProjectTypeList />
    </ProtectedRoute>
  ),
});

// プロジェクトタイプ詳細（ステータス定義一覧を含む）ルート（認証ガード付き）
const projectTypeDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/project-types/$projectTypeId/status-definitions',
  component: () => {
    const { projectTypeId } = projectTypeDetailRoute.useParams();
    // H-12 監査対応: 詳細画面も認証済みユーザーのみアクセス可能
    return (
      <ProtectedRoute fallback={<AuthRequired />}>
        <ProjectTypeDetail projectTypeId={projectTypeId} />
      </ProtectedRoute>
    );
  },
});

// ステータス定義のバージョン履歴ルート（認証ガード付き）
const statusDefinitionVersionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/status-definitions/$statusDefinitionId/versions',
  component: () => {
    const { statusDefinitionId } = statusDefinitionVersionsRoute.useParams();
    // H-12 監査対応: バージョン履歴画面も認証済みユーザーのみアクセス可能
    return (
      <ProtectedRoute fallback={<AuthRequired />}>
        <StatusDefinitionDetail statusDefinitionId={statusDefinitionId} />
      </ProtectedRoute>
    );
  },
});

// テナント拡張一覧ルート（認証ガード付き）
const tenantExtensionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenants/$tenantId/extensions',
  component: () => {
    const { tenantId } = tenantExtensionsRoute.useParams();
    // H-12 監査対応: テナント拡張管理は認証済みユーザーのみアクセス可能
    return (
      <ProtectedRoute fallback={<AuthRequired />}>
        <TenantExtensionList tenantId={tenantId} />
      </ProtectedRoute>
    );
  },
});

// テナント拡張編集ルート（認証ガード付き）
const tenantExtensionEditRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tenants/$tenantId/extensions/$statusDefinitionId',
  component: () => {
    const { tenantId, statusDefinitionId } = tenantExtensionEditRoute.useParams();
    // H-12 監査対応: テナント拡張編集も認証済みユーザーのみアクセス可能
    return (
      <ProtectedRoute fallback={<AuthRequired />}>
        <TenantExtensionForm tenantId={tenantId} statusDefinitionId={statusDefinitionId} />
      </ProtectedRoute>
    );
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
