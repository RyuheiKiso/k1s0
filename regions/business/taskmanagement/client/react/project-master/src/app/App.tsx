import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { queryClient } from '../lib/query-client';
import { router } from './router';
// H-12 監査対応: system-client パッケージの AuthProvider・ErrorBoundary を使用して認証基盤を統一する
// インライン実装から共通実装へ移行し、本番環境でのエラー詳細隠蔽や認証状態管理を一元化する
import { AuthProvider, ErrorBoundary } from '@k1s0/system-client';

// アプリケーションのルートコンポーネント
// ErrorBoundary: レンダリングエラーをキャッチしてフォールバックUIを表示（本番環境ではエラー詳細を隠蔽）
// AuthProvider: JWT 認証状態を管理し、子コンポーネントに認証コンテキストを提供
// QueryClientProvider: TanStack Queryのキャッシュとデータ取得を提供
// RouterProvider: TanStack Routerのルーティングを提供
export function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        {/* H-12 監査対応: BFF エンドポイント経由で認証状態を管理する */}
        <AuthProvider apiBaseURL="/bff">
          <RouterProvider router={router} />
        </AuthProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
