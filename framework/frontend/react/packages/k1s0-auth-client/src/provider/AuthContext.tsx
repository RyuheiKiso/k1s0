import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  useMemo,
  useRef,
  type ReactNode,
} from 'react';
import type {
  AuthState,
  AuthUser,
  AuthError,
  AuthClientConfig,
  TokenPair,
} from '../types.js';
import { TokenManager } from '../token/TokenManager.js';
import { SessionTokenStorage } from '../token/storage.js';
import { decodeToken, claimsToUser } from '../token/decoder.js';

/**
 * 認証コンテキストの値
 */
export interface AuthContextValue {
  /** 現在の認証状態 */
  state: AuthState;
  /** トークンマネージャーへの参照 */
  tokenManager: TokenManager;
  /** ログイン処理（トークンを保存） */
  login: (tokens: TokenPair) => void;
  /** ログアウト処理（トークンをクリア） */
  logout: () => void;
  /** トークンリフレッシュを強制実行 */
  refreshToken: () => Promise<boolean>;
  /** 認証エラーハンドラ（401受信時に呼ばれる） */
  handleAuthError: (error?: AuthError) => void;
  /** 権限チェック（ロール） */
  hasRole: (role: string) => boolean;
  /** 権限チェック（パーミッション） */
  hasPermission: (permission: string) => boolean;
  /** 複数ロールのいずれかを持つか */
  hasAnyRole: (roles: string[]) => boolean;
  /** 複数パーミッションのすべてを持つか */
  hasAllPermissions: (permissions: string[]) => boolean;
}

const AuthContext = createContext<AuthContextValue | null>(null);

interface AuthProviderProps {
  children: ReactNode;
  config?: AuthClientConfig;
}

/**
 * 認証コンテキストプロバイダ
 *
 * - トークンの永続化と状態管理
 * - 自動リフレッシュ（設定時）
 * - 権限チェック機能
 */
export function AuthProvider({ children, config }: AuthProviderProps) {
  const [state, setState] = useState<AuthState>({
    status: 'loading',
    isLoading: true,
    isAuthenticated: false,
  });

  // TokenManager は一度だけ作成
  const tokenManagerRef = useRef<TokenManager | null>(null);
  if (!tokenManagerRef.current) {
    tokenManagerRef.current = new TokenManager({
      storage: config?.storage ?? new SessionTokenStorage(),
      refreshToken: config?.refreshToken,
      refreshMarginMs: config?.refreshMarginMs,
      autoRefresh: config?.autoRefresh ?? true,
      onRefreshError: (error) => {
        setState({
          status: 'unauthenticated',
          isLoading: false,
          isAuthenticated: false,
          error,
        });
        config?.onAuthError?.(error);
      },
    });
  }
  const tokenManager = tokenManagerRef.current;

  // 初回マウント時にトークンを確認
  useEffect(() => {
    const tokens = tokenManager.getTokens();

    if (tokens) {
      const result = decodeToken(tokens.accessToken);
      if (result.success) {
        const user = claimsToUser(result.claims);
        setState({
          status: 'authenticated',
          user,
          isLoading: false,
          isAuthenticated: true,
        });

        // 自動リフレッシュのスケジューリングを開始
        if (config?.autoRefresh !== false && tokens.refreshToken) {
          tokenManager.setTokens(tokens);
        }
      } else {
        setState({
          status: 'unauthenticated',
          isLoading: false,
          isAuthenticated: false,
        });
      }
    } else {
      setState({
        status: 'unauthenticated',
        isLoading: false,
        isAuthenticated: false,
      });
    }

    // リフレッシュイベントをリッスン
    const unsubscribe = tokenManager.onRefresh((newTokens) => {
      if (newTokens) {
        const result = decodeToken(newTokens.accessToken);
        if (result.success) {
          const user = claimsToUser(result.claims);
          setState({
            status: 'authenticated',
            user,
            isLoading: false,
            isAuthenticated: true,
          });
        }
      } else {
        setState({
          status: 'unauthenticated',
          isLoading: false,
          isAuthenticated: false,
        });
      }
    });

    return () => {
      unsubscribe();
    };
  }, [tokenManager, config?.autoRefresh]);

  // クリーンアップ
  useEffect(() => {
    return () => {
      tokenManager.dispose();
    };
  }, [tokenManager]);

  const login = useCallback(
    (tokens: TokenPair) => {
      const result = decodeToken(tokens.accessToken);
      if (result.success) {
        tokenManager.setTokens(tokens);
        const user = claimsToUser(result.claims);
        setState({
          status: 'authenticated',
          user,
          isLoading: false,
          isAuthenticated: true,
        });
      } else {
        setState({
          status: 'error',
          error: result.error,
          isLoading: false,
          isAuthenticated: false,
        });
      }
    },
    [tokenManager]
  );

  const logout = useCallback(() => {
    tokenManager.clearTokens();
    setState({
      status: 'unauthenticated',
      isLoading: false,
      isAuthenticated: false,
    });
    config?.onLogout?.();
  }, [tokenManager, config]);

  const refreshToken = useCallback(async (): Promise<boolean> => {
    const newTokens = await tokenManager.forceRefresh();
    return newTokens !== null;
  }, [tokenManager]);

  const handleAuthError = useCallback(
    (error?: AuthError) => {
      tokenManager.clearTokens();
      const authError: AuthError = error ?? {
        code: 'UNAUTHORIZED',
        message: 'Authentication required',
      };
      setState({
        status: 'unauthenticated',
        error: authError,
        isLoading: false,
        isAuthenticated: false,
      });
      config?.onAuthError?.(authError);
    },
    [tokenManager, config]
  );

  const hasRole = useCallback(
    (role: string): boolean => {
      return state.user?.roles.includes(role) ?? false;
    },
    [state.user]
  );

  const hasPermission = useCallback(
    (permission: string): boolean => {
      return state.user?.permissions.includes(permission) ?? false;
    },
    [state.user]
  );

  const hasAnyRole = useCallback(
    (roles: string[]): boolean => {
      if (!state.user) return false;
      return roles.some((role) => state.user!.roles.includes(role));
    },
    [state.user]
  );

  const hasAllPermissions = useCallback(
    (permissions: string[]): boolean => {
      if (!state.user) return false;
      return permissions.every((p) => state.user!.permissions.includes(p));
    },
    [state.user]
  );

  const value = useMemo<AuthContextValue>(
    () => ({
      state,
      tokenManager,
      login,
      logout,
      refreshToken,
      handleAuthError,
      hasRole,
      hasPermission,
      hasAnyRole,
      hasAllPermissions,
    }),
    [
      state,
      tokenManager,
      login,
      logout,
      refreshToken,
      handleAuthError,
      hasRole,
      hasPermission,
      hasAnyRole,
      hasAllPermissions,
    ]
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
  return useAuth().state.isAuthenticated;
}

/**
 * 認証済みユーザー情報を取得するフック
 */
export function useUser(): AuthUser | undefined {
  return useAuth().state.user;
}

/**
 * 権限チェックフック
 */
export function usePermissions() {
  const { hasRole, hasPermission, hasAnyRole, hasAllPermissions } = useAuth();
  return { hasRole, hasPermission, hasAnyRole, hasAllPermissions };
}
