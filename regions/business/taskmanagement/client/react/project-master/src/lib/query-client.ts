import { QueryClient } from '@tanstack/react-query';

// TanStack Query のデフォルト設定（M-018）
// staleTime の使い分けガイド:
//   - リアルタイム性が重要なデータ（最新マスタ等）: useQuery の staleTime を 0 にして即時再フェッチ
//   - 比較的静的なデータ（参照系マスタ等）: 5〜30分の staleTime でサーバー負荷を軽減
//   - ここで設定するのはデフォルト値（全クエリへのフォールバック）
// staleTime: 5分間はキャッシュを新鮮とみなす（デフォルト: 0 = 常にstale）
// retry: 失敗時に1回だけリトライ（デフォルト: 3 回）
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 1,
    },
  },
});
