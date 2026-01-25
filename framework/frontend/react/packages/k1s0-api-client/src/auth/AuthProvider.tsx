import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  useMemo,
  type ReactNode,
} from 'react';
import type { AuthState, AuthConfig, TokenPair } from './types.js';
import { TokenManager, SessionTokenStorage } from './TokenManager.js';

interface AuthContextValue {
  /** 現在の認証状態 */
  state: AuthState;
  /** トークンマネージャーへの参照（APIクライアントで使用） */
  tokenManager: TokenManager;
  /** ログイン処理（トークンを保存） */
  login: (tokens: TokenPair) => void;
  /** ログアウト処理（トークンをクリア） */
  logout: () => void;
  /** 認証エラーハンドラ（401受信時に呼ばれる） */
  handleAuthError: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

interface AuthProviderProps {
  children: ReactNode;
  config?: AuthConfig;
}

/**
 * 認証コンテキストプロバイダ
 * - トークンの永続化と状態管理
 * - 自動リフレッシュ（設定時）
 * - APIクライアントとの連携
 */
export function AuthProvider({ children, config }: AuthProviderProps) {
  const [state, setState] = useState<AuthState>({ status: 'loading' });

  const tokenManager = useMemo(() => {
    return new TokenManager({
      storage: config?.storage ?? new SessionTokenStorage(),
      refreshToken: config?.refreshToken,
      refreshMarginMs: config?.refreshMarginMs,
    });
  }, [config?.storage, config?.refreshToken, config?.refreshMarginMs]);

  // 初回マウント時にトークンを確認
  useEffect(() => {
    const tokens = tokenManager.getTokens();
    if (tokens && tokenManager.isTokenValid(tokens)) {
      setState({ status: 'authenticated', accessToken: tokens.accessToken });
    } else {
      setState({ status: 'unauthenticated' });
    }
  }, [tokenManager]);

  const login = useCallback(
    (tokens: TokenPair) => {
      tokenManager.setTokens(tokens);
      setState({ status: 'authenticated', accessToken: tokens.accessToken });
    },
    [tokenManager]
  );

  const logout = useCallback(() => {
    tokenManager.clearTokens();
    setState({ status: 'unauthenticated' });
  }, [tokenManager]);

  const handleAuthError = useCallback(() => {
    tokenManager.clearTokens();
    setState({ status: 'unauthenticated' });
    config?.onAuthError?.();
  }, [tokenManager, config]);

  const value = useMemo<AuthContextValue>(
    () => ({
      state,
      tokenManager,
      login,
      logout,
      handleAuthError,
    }),
    [state, tokenManager, login, logout, handleAuthError]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/**
 * 認証コンテキストを取得するフック
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

/**
 * 認証状態のみを取得するフック（軽量版）
 */
export function useAuthState(): AuthState {
  return useAuth().state;
}

/**
 * 認証済みかどうかを判定するフック
 */
export function useIsAuthenticated(): boolean {
  const state = useAuthState();
  return state.status === 'authenticated';
}
