import { useState, useCallback, useRef, useEffect } from 'react';
import type { ApiResponse, RequestOptions } from './types.js';
import { useApiClient } from './ApiClientProvider.js';
import { ApiError } from '../error/ApiError.js';

/**
 * APIリクエストの状態
 */
export type ApiRequestState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T; response: ApiResponse<T> }
  | { status: 'error'; error: ApiError };

/**
 * useApiRequestの戻り値
 */
export interface UseApiRequestResult<T> {
  /** 現在の状態 */
  state: ApiRequestState<T>;
  /** リクエストを実行 */
  execute: () => Promise<T>;
  /** 状態をリセット */
  reset: () => void;
  /** ローディング中かどうか */
  isLoading: boolean;
  /** エラーがあるかどうか */
  isError: boolean;
  /** 成功したかどうか */
  isSuccess: boolean;
  /** データ（成功時のみ） */
  data: T | undefined;
  /** エラー（エラー時のみ） */
  error: ApiError | undefined;
}

/**
 * APIリクエストを実行するフック
 * @param path APIパス
 * @param options リクエストオプション
 */
export function useApiRequest<T>(
  path: string,
  options?: RequestOptions
): UseApiRequestResult<T> {
  const client = useApiClient();
  const [state, setState] = useState<ApiRequestState<T>>({ status: 'idle' });
  const abortControllerRef = useRef<AbortController | null>(null);

  // アンマウント時にリクエストをキャンセル
  useEffect(() => {
    return () => {
      abortControllerRef.current?.abort();
    };
  }, []);

  const execute = useCallback(async (): Promise<T> => {
    // 前のリクエストをキャンセル
    abortControllerRef.current?.abort();
    abortControllerRef.current = new AbortController();

    setState({ status: 'loading' });

    try {
      const response = await client.request<T>(path, {
        ...options,
        signal: abortControllerRef.current.signal,
      });
      setState({ status: 'success', data: response.data, response });
      return response.data;
    } catch (error) {
      const apiError = ApiError.from(error);
      setState({ status: 'error', error: apiError });
      throw apiError;
    }
  }, [client, path, options]);

  const reset = useCallback(() => {
    abortControllerRef.current?.abort();
    setState({ status: 'idle' });
  }, []);

  return {
    state,
    execute,
    reset,
    isLoading: state.status === 'loading',
    isError: state.status === 'error',
    isSuccess: state.status === 'success',
    data: state.status === 'success' ? state.data : undefined,
    error: state.status === 'error' ? state.error : undefined,
  };
}

/**
 * ミューテーション（POST/PUT/PATCH/DELETE）用フック
 */
export interface UseMutationResult<TData, TVariables> {
  /** 現在の状態 */
  state: ApiRequestState<TData>;
  /** ミューテーションを実行 */
  mutate: (variables: TVariables) => Promise<TData>;
  /** 状態をリセット */
  reset: () => void;
  /** ローディング中かどうか */
  isLoading: boolean;
  /** エラーがあるかどうか */
  isError: boolean;
  /** 成功したかどうか */
  isSuccess: boolean;
  /** データ（成功時のみ） */
  data: TData | undefined;
  /** エラー（エラー時のみ） */
  error: ApiError | undefined;
}

/**
 * ミューテーション用フック
 * @param mutationFn ミューテーション関数
 */
export function useMutation<TData, TVariables>(
  mutationFn: (variables: TVariables) => Promise<TData>
): UseMutationResult<TData, TVariables> {
  const [state, setState] = useState<ApiRequestState<TData>>({ status: 'idle' });

  const mutate = useCallback(
    async (variables: TVariables): Promise<TData> => {
      setState({ status: 'loading' });

      try {
        const data = await mutationFn(variables);
        setState({
          status: 'success',
          data,
          response: { data, status: 200, headers: new Headers() },
        });
        return data;
      } catch (error) {
        const apiError = ApiError.from(error);
        setState({ status: 'error', error: apiError });
        throw apiError;
      }
    },
    [mutationFn]
  );

  const reset = useCallback(() => {
    setState({ status: 'idle' });
  }, []);

  return {
    state,
    mutate,
    reset,
    isLoading: state.status === 'loading',
    isError: state.status === 'error',
    isSuccess: state.status === 'success',
    data: state.status === 'success' ? state.data : undefined,
    error: state.status === 'error' ? state.error : undefined,
  };
}
