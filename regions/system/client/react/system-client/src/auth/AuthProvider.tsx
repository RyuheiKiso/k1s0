import { useState, useEffect, useCallback, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';
import { createApiClient, setCsrfToken } from '../http/apiClient';
import { navigateTo } from './navigation';

interface AuthProviderProps {
  children: ReactNode;
  apiBaseURL?: string;
}

// BFF /auth/session のレスポンス型
interface SessionResponse {
  id: string;
  authenticated: boolean;
  csrf_token: string;
}

export function AuthProvider({ children, apiBaseURL = '/bff' }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // 401 未認証エラー時にログインページへリダイレクトするコールバックを設定
  const apiClient = createApiClient({
    baseURL: apiBaseURL,
    onUnauthorized: () => { navigateTo(`${apiBaseURL}/auth/login`); },
  });

  // 初期化時にセッション確認（BFF の /auth/session エンドポイントを使用）
  useEffect(() => {
    const checkSession = async () => {
      try {
        const response = await apiClient.get<SessionResponse>('/auth/session');
        const data = response.data;
        if (data.authenticated) {
          // CSRF トークンを apiClient に設定し、以降のリクエストで自動送信する
          setCsrfToken(data.csrf_token);
          setUser({
            id: data.id,
            username: data.id,
            roles: [],
          });
        } else {
          setUser(null);
        }
      } catch {
        setUser(null);
      } finally {
        setLoading(false);
      }
    };

    checkSession();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // BFF の OAuth2/OIDC 認可コードフローへリダイレクトする
  const login = useCallback(() => {
    navigateTo(`${apiBaseURL}/auth/login`);
  }, [apiBaseURL]);

  // ログアウト時に CSRF トークンもクリアする
  const logout = useCallback(async () => {
    await apiClient.post('/auth/logout');
    setCsrfToken(null);
    setUser(null);
  }, [apiClient]);

  if (loading) {
    return null;
  }

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: user !== null,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
