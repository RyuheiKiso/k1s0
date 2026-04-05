import { lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
// FE-03 対応: インライン ErrorBoundary を廃止し、system-client 共通コンポーネントを使用する
// system-client の ErrorBoundary は本番環境でエラー詳細を隠蔽する import.meta.env.DEV 分岐済み
// AccessDenied を追加: 権限不足時のフォールバック表示に使用（M-27 監査対応）
import { AuthProvider, ProtectedRoute, AccessDenied, ErrorBoundary } from './lib/systemClient';
// FE-004 監査対応: BFF ベース URL をハードコードせず YAML 設定ファイルから取得する
import { appConfig } from './config';

// ページコンポーネントの遅延読み込み（コード分割）
const AppListPage = lazy(() => import('./pages/AppListPage').then((m) => ({ default: m.AppListPage })));
const AppDetailPage = lazy(() => import('./pages/AppDetailPage').then((m) => ({ default: m.AppDetailPage })));
const AdminPage = lazy(() => import('./pages/AdminPage').then((m) => ({ default: m.AdminPage })));

// TanStack Queryクライアントの設定
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 1,
    },
  },
});

// アプリケーションのルートコンポーネント
export function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        {/* FE-004 監査対応: BFF ベース URL は config.yaml の bff.base_url から取得する */}
        <AuthProvider apiBaseURL={appConfig.bff.base_url}>
          <BrowserRouter>
            <header className="app-header">
              <nav className="app-header__nav" aria-label="メインナビゲーション">
                <Link to="/" className="app-header__logo">
                  App Catalog
                </Link>
                <Link to="/admin">管理</Link>
              </nav>
            </header>
            <main className="app-main">
              {/* Suspenseでルートコンポーネントの遅延読み込みを処理 */}
              <Suspense fallback={<div>読み込み中...</div>}>
                <Routes>
                  <Route path="/" element={<AppListPage />} />
                  <Route path="/apps/:appId" element={<AppDetailPage />} />
                  {/* AccessDenied を使用: 権限不足時にスピナーが永続表示されるのを防ぐ（M-27 監査対応） */}
                  <Route
                    path="/admin"
                    element={
                      <ProtectedRoute
                        requiredRoles={['admin']}
                        fallback={<AccessDenied message="このページは管理者アカウントのみアクセスできます。" />}
                      >
                        <AdminPage />
                      </ProtectedRoute>
                    }
                  />
                </Routes>
              </Suspense>
            </main>
          </BrowserRouter>
        </AuthProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
