import { lazy, Suspense, Component, type ReactNode } from 'react';
import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider, ProtectedRoute, LoadingSpinner } from './lib/systemClient';

// ページコンポーネントの遅延読み込み（コード分割）
const AppListPage = lazy(() => import('./pages/AppListPage').then((m) => ({ default: m.AppListPage })));
const AppDetailPage = lazy(() => import('./pages/AppDetailPage').then((m) => ({ default: m.AppDetailPage })));
const AdminPage = lazy(() => import('./pages/AdminPage').then((m) => ({ default: m.AdminPage })));

// ErrorBoundaryのProps定義
interface ErrorBoundaryProps {
  children: ReactNode;
}

// ErrorBoundaryの内部状態
interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

// エラーバウンダリコンポーネント: 子コンポーネントのレンダリングエラーをキャッチして表示
class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  // レンダリングエラー発生時にエラー状態を更新
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  override render() {
    // エラー発生時はフォールバックUIを表示
    if (this.state.hasError) {
      return (
        <div role="alert">
          <h2>エラーが発生しました</h2>
          <p>{this.state.error?.message}</p>
        </div>
      );
    }
    return this.props.children;
  }
}

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
        <AuthProvider apiBaseURL="/bff">
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
                  <Route
                    path="/admin"
                    element={
                      <ProtectedRoute
                        requiredRoles={['admin']}
                        fallback={<LoadingSpinner message="認証済みの管理者アカウントが必要です" />}
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
