import { useState, useEffect, useCallback, useMemo, type ReactNode } from 'react';
import { AuthContext, type User } from './AuthContext';
import { createApiClient, setCsrfToken } from '../http/apiClient';
import { navigateTo } from './navigation';
// ローディング中のスピナー表示に使用する既存コンポーネントをインポートする
import { LoadingSpinner } from '../components/LoadingSpinner';

/**
 * L-003/L-016 監査対応: unknown 型の catch 例外から HTTP ステータスコードを安全に取り出すヘルパー。
 * TypeScript の catch(e) は e が unknown 型のため、直接プロパティアクセスは型安全でない。
 * typeof/instanceof チェックを通じて安全に値を取得し、危険な型アサーション (as SomeType) を最小化する。
 * Record<string, unknown> へのキャストは、in 演算子による存在確認後の安全な narrowing として使用する。
 */
function extractHttpStatus(error: unknown): number | undefined {
  if (typeof error !== 'object' || error === null) {
    return undefined;
  }
  // 'response' キーの存在を確認してから型を narrowing する
  if (!('response' in error)) {
    return undefined;
  }
  const asRecord = error as Record<string, unknown>;
  const response = asRecord['response'];
  if (typeof response !== 'object' || response === null) {
    return undefined;
  }
  const status = (response as Record<string, unknown>)['status'];
  return typeof status === 'number' ? status : undefined;
}

interface AuthProviderProps {
  children: ReactNode;
  // BFF のベース URL（必須）。呼び出し側アプリの設定ファイル（config.yaml 等）から取得して渡すこと（FE-004 監査対応）
  // ハードコードを禁止し、呼び出し元が必ず設定から渡すよう強制する
  apiBaseURL: string;
}

// BFF /auth/session のレスポンス型
interface SessionResponse {
  id: string;
  authenticated: boolean;
  csrf_token: string;
  // ユーザーのロール一覧（admin 等の権限管理に使用）
  roles?: string[];
}

export function AuthProvider({ children, apiBaseURL }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // 相対パス検証: 外部URLによるオープンリダイレクト防止（M-28 監査対応）
  // '/' で始まらない値（http:// 等の外部URLを含む）はルートパスにフォールバックする
  // apiBaseURL は呼び出し元が必ず渡す必須 prop のため、フォールバックは最終セーフティネット
  const safeApiBaseURL = apiBaseURL.startsWith('/') ? apiBaseURL : '/';

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
      } catch (error) {
        // MED-007 監査対応: HTTP ステータスコードに応じてエラーハンドリングを分岐する。
        // L-003/L-016 監査対応: (error as {...}) の型アサーションを排除し、extractHttpStatus() で安全にステータスを取得する。
        const status = extractHttpStatus(error);
        if (status === 403) {
          // 403 Forbidden: 認可エラーのためユーザー情報は保持し、警告ログを出力する
          console.warn('[k1s0-auth] セッション確認で 403 Forbidden が返されました。権限が不足しています。');
        } else if (status !== undefined && status >= 500) {
          // 5xx Server Error: サービス障害のためユーザー情報は保持し、警告ログを出力する
          console.warn(`[k1s0-auth] セッション確認でサーバーエラー (${status}) が返されました。サービスが一時的に利用できません。`);
        } else {
          // 401 未認証・ネットワークエラー・その他: ユーザー情報をクリアしてログアウト状態にする
          setUser(null);
        }
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
    } catch (error) {
      // MED-007 監査対応: ログアウト時もステータスコードに応じてログを分岐する。
      // L-003/L-016 監査対応: (error as {...}) の型アサーションを排除し、extractHttpStatus() で安全にステータスを取得する。
      const status = extractHttpStatus(error);
      if (status !== undefined && status >= 500) {
        // 5xx Server Error: サービス障害を警告ログに記録する
        console.warn(`[k1s0-auth] ログアウトリクエストでサーバーエラー (${status}) が返されました。`);
      }
      // それ以外（ネットワークエラー等）はサイレントに処理する（finally でクリア済み）
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
        // M-009 監査対応: ローディング状態をコンテキストに公開する
        // ProtectedRoute が使用して fallback フラッシュを防止する
        loading,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
