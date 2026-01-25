import React, { type ReactNode } from 'react';
import { Box, CircularProgress, Skeleton } from '@mui/material';
import type { ApiRequestState } from '../client/useApiRequest.js';
import { ErrorDisplay } from './ErrorDisplay.js';
import { ApiError } from '../error/ApiError.js';

interface AsyncContentProps<T> {
  /** APIリクエストの状態 */
  state: ApiRequestState<T>;
  /** 成功時のレンダリング関数 */
  children: (data: T) => ReactNode;
  /** ローディング時のカスタムUI */
  loading?: ReactNode;
  /** エラー時のカスタムUI */
  error?: ReactNode | ((error: ApiError, retry?: () => void) => ReactNode);
  /** リトライ関数 */
  onRetry?: () => void;
  /** アイドル時のUI */
  idle?: ReactNode;
  /** エラー詳細を表示するか */
  showErrorDetails?: boolean;
}

/**
 * APIリクエストの状態に応じてコンテンツを切り替えるコンポーネント
 */
export function AsyncContent<T>({
  state,
  children,
  loading,
  error,
  onRetry,
  idle,
  showErrorDetails = false,
}: AsyncContentProps<T>) {
  switch (state.status) {
    case 'idle':
      return <>{idle ?? null}</>;

    case 'loading':
      return (
        <>
          {loading ?? (
            <Box
              sx={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                py: 4,
              }}
            >
              <CircularProgress />
            </Box>
          )}
        </>
      );

    case 'error':
      if (error) {
        return (
          <>
            {typeof error === 'function'
              ? error(state.error, onRetry)
              : error}
          </>
        );
      }
      return (
        <ErrorDisplay
          error={state.error}
          onRetry={onRetry}
          showDetails={showErrorDetails}
        />
      );

    case 'success':
      return <>{children(state.data)}</>;
  }
}

interface DataLoaderProps<T> {
  /** データ（undefinedの場合はローディング） */
  data: T | undefined;
  /** エラー */
  error: ApiError | undefined;
  /** ローディング中かどうか */
  isLoading: boolean;
  /** 成功時のレンダリング関数 */
  children: (data: T) => ReactNode;
  /** ローディング時のカスタムUI */
  loading?: ReactNode;
  /** エラー時のカスタムUI */
  errorComponent?: ReactNode;
  /** リトライ関数 */
  onRetry?: () => void;
}

/**
 * シンプルなデータローダーコンポーネント
 */
export function DataLoader<T>({
  data,
  error,
  isLoading,
  children,
  loading,
  errorComponent,
  onRetry,
}: DataLoaderProps<T>) {
  if (isLoading) {
    return (
      <>
        {loading ?? (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
            <CircularProgress />
          </Box>
        )}
      </>
    );
  }

  if (error) {
    return <>{errorComponent ?? <ErrorDisplay error={error} onRetry={onRetry} />}</>;
  }

  if (data !== undefined) {
    return <>{children(data)}</>;
  }

  return null;
}

/**
 * スケルトンローダー
 */
export function SkeletonLoader({
  isLoading,
  children,
  lines = 3,
  variant = 'text',
}: {
  isLoading: boolean;
  children: ReactNode;
  lines?: number;
  variant?: 'text' | 'rectangular' | 'rounded' | 'circular';
}) {
  if (isLoading) {
    return (
      <Box>
        {Array.from({ length: lines }).map((_, index) => (
          <Skeleton key={index} variant={variant} sx={{ mb: 1 }} />
        ))}
      </Box>
    );
  }

  return <>{children}</>;
}
