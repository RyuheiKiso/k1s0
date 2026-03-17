import { Component, type ReactNode } from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { queryClient } from '../lib/query-client';
import { router } from './router';

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
