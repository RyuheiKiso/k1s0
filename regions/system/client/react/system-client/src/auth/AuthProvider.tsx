import { useState, useEffect, useCallback, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';
import { createApiClient } from '../http/apiClient';

interface AuthProviderProps {
  children: ReactNode;
  apiBaseURL?: string;
}

export function AuthProvider({ children, apiBaseURL = '/bff' }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  const apiClient = createApiClient({ baseURL: apiBaseURL });

  // 初期化時にセッション確認
  useEffect(() => {
    const checkSession = async () => {
      try {
        const response = await apiClient.get<User>('/auth/me');
        setUser(response.data);
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
      const response = await apiClient.post<User>('/auth/login', {
        username,
        password,
      });
      setUser(response.data);
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
