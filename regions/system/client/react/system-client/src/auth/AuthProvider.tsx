import { useState, useEffect, useCallback, useMemo, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';
import { createApiClient, setCsrfToken } from '../http/apiClient';
import { navigateTo } from './navigation';
// ローディング中のスピナー表示に使用する既存コンポーネントをインポートする
import { LoadingSpinner } from '../components/LoadingSpinner';

interface AuthProviderProps {
  children: ReactNode;
  apiBaseURL?: string;
}

// BFF /auth/session のレスポンス型
interface SessionResponse {
  id: string;
  authenticated: boolean;
  csrf_token: string;
  // ユーザーのロール一覧（admin 等の権限管理に使用）
  roles?: string[];
}

export function AuthProvider({ children, apiBaseURL = '/bff' }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // 相対パス検証: 外部URLによるオープンリダイレクト防止（M-28 監査対応）
  // '/' で始まらない値（http:// 等の外部URLを含む）はデフォルト値にフォールバックする
  const safeApiBaseURL = apiBaseURL.startsWith('/') ? apiBaseURL : '/bff';

  // APIクライアントをメモ化し、safeApiBaseURL が変わらない限り再生成しない（レンダーごとの無駄な生成を防止）
  const apiClient = useMemo(() => createApiClient({
    baseURL: safeApiBaseURL,
    onUnauthorized: () => { navigateTo(`${safeApiBaseURL}/auth/login`); },
  }), [safeApiBaseURL]);

  // 初期化時にセッション確認（BFF の /auth/session エンドポイントを使用）
  // apiClient は useMemo でメモ化されているため、apiBaseURL が変わらない限り再実行されない（M-13 監査対応）
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
            roles: data.roles ?? [],
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
  }, [apiClient]);

  // BFF の OAuth2/OIDC 認可コードフローへリダイレクトする（safeApiBaseURL で外部URL混入を防止）
  const login = useCallback(() => {
    navigateTo(`${safeApiBaseURL}/auth/login`);
  }, [safeApiBaseURL]);

  // ログアウト時に CSRF トークンもクリアする
  // ネットワークエラーが発生してもクライアント側の認証状態は必ずクリアする（finally で保証）
  const logout = useCallback(async () => {
    try {
      await apiClient.post('/auth/logout');
    } catch {
      // サーバー側のログアウトが失敗してもクライアント側のセッションは必ずクリアする
    } finally {
      setCsrfToken(null);
      setUser(null);
    }
  }, [apiClient]);

  // セッション確認が完了するまでローディングスピナーを表示し、null を返してコンテンツが消えるのを防ぐ
  if (loading) {
    return <LoadingSpinner message="認証情報を確認中..." />;
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
