import { useState, useEffect, useCallback, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';
import { createApiClient } from '../http/apiClient';

interface AuthProviderProps {
  children: ReactNode;
  apiBaseURL?: string;
}

interface UserResponse {
  id: string;
  username: string;
  roles?: string[];
  realm_access?: {
    roles?: string[];
  };
}

function normalizeUser(user: UserResponse): User {
  return {
    id: user.id,
    username: user.username,
    roles: user.roles ?? user.realm_access?.roles ?? [],
  };
}

export function AuthProvider({ children, apiBaseURL = '/bff' }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // 401 未認証エラー時にログインページへリダイレクトするコールバックを設定
  const apiClient = createApiClient({
    baseURL: apiBaseURL,
    onUnauthorized: () => { window.location.href = '/auth/login'; },
  });

  // 初期化時にセッション確認
  useEffect(() => {
    const checkSession = async () => {
      try {
        const response = await apiClient.get<UserResponse>('/auth/me');
        setUser(normalizeUser(response.data));
      } catch {
        setUser(null);
      } finally {
        setLoading(false);
      }
    };

    checkSession();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const login = useCallback(
    async ({ username, password }: { username: string; password: string }) => {
      const response = await apiClient.post<UserResponse>('/auth/login', {
        username,
        password,
      });
      setUser(normalizeUser(response.data));
    },
    [apiClient],
  );

  const logout = useCallback(async () => {
    await apiClient.post('/auth/logout');
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
