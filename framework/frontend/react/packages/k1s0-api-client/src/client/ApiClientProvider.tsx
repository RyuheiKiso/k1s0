import React, {
  createContext,
  useContext,
  useMemo,
  type ReactNode,
} from 'react';
import { ApiClient, createApiClient } from './ApiClient.js';
import type { ApiClientConfig } from './types.js';
import { useAuth } from '../auth/AuthProvider.js';
import { defaultTelemetry } from '../telemetry/OTelTracer.js';

const ApiClientContext = createContext<ApiClient | null>(null);

interface ApiClientProviderProps {
  children: ReactNode;
  /** APIクライアントの設定 */
  config: Omit<ApiClientConfig, 'tokenManager' | 'onAuthError'>;
}

/**
 * APIクライアントプロバイダ
 * AuthProviderの内側で使用すること
 * - 認証トークンの自動付与
 * - 認証エラー時の自動ハンドリング
 */
export function ApiClientProvider({ children, config }: ApiClientProviderProps) {
  const { tokenManager, handleAuthError } = useAuth();

  const client = useMemo(() => {
    return createApiClient({
      ...config,
      tokenManager,
      telemetry: config.telemetry ?? defaultTelemetry,
      onAuthError: handleAuthError,
    });
  }, [config, tokenManager, handleAuthError]);

  return (
    <ApiClientContext.Provider value={client}>
      {children}
    </ApiClientContext.Provider>
  );
}

/**
 * APIクライアントを取得するフック
 */
export function useApiClient(): ApiClient {
  const client = useContext(ApiClientContext);
  if (!client) {
    throw new Error('useApiClient must be used within an ApiClientProvider');
  }
  return client;
}

/**
 * 認証なしのAPIクライアントプロバイダ
 * AuthProviderなしで使用可能（公開API等）
 */
export function PublicApiClientProvider({
  children,
  config,
}: {
  children: ReactNode;
  config: ApiClientConfig;
}) {
  const client = useMemo(() => {
    return createApiClient({
      ...config,
      telemetry: config.telemetry ?? defaultTelemetry,
    });
  }, [config]);

  return (
    <ApiClientContext.Provider value={client}>
      {children}
    </ApiClientContext.Provider>
  );
}
