import { QueryClient } from '@tanstack/react-query';

// TanStack Query のデフォルト設定
// staleTime: 5分間はキャッシュを新鮮とみなす
// retry: 失敗時に1回だけリトライ
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 1,
    },
  },
});
