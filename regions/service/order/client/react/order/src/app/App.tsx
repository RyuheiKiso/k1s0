import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { ErrorBoundary } from '../components/ErrorBoundary';
import { queryClient } from '../lib/query-client';
import { router } from './router';

// アプリケーションのルートコンポーネント
// ErrorBoundary: レンダリングエラーをキャッチしてフォールバックUIを表示
// QueryClientProvider: TanStack Queryのキャッシュとデータ取得を提供
// RouterProvider: TanStack Routerのルーティングを提供
export function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
