import React, { useEffect, useState, type ReactNode } from 'react';
import { useAuth } from '../provider/AuthContext.js';
import type { AuthGuardConfig, AuthUser } from '../types.js';

interface AuthGuardProps extends AuthGuardConfig {
  children: ReactNode;
  /** 認証チェック中に表示するコンポーネント */
  loadingComponent?: ReactNode;
  /** 認証されていない場合に表示するコンポーネント（redirectTo が設定されていない場合） */
  unauthenticatedComponent?: ReactNode;
  /** 権限不足の場合に表示するコンポーネント（forbiddenRedirectTo が設定されていない場合） */
  forbiddenComponent?: ReactNode;
  /** リダイレクト関数（react-router の navigate 等）*/
  navigate?: (to: string) => void;
}

/**
 * 認証・認可ガードコンポーネント
 *
 * - 認証状態をチェックし、未認証の場合はリダイレクトまたはコンポーネント表示
 * - ロール/パーミッションをチェックし、権限不足の場合はリダイレクトまたはコンポーネント表示
 * - カスタム認可関数をサポート
 */
export function AuthGuard({
  children,
  roles,
  permissions,
  redirectTo,
  forbiddenRedirectTo,
  authorize,
  loadingComponent,
  unauthenticatedComponent,
  forbiddenComponent,
  navigate,
}: AuthGuardProps) {
  const { state, hasAnyRole, hasAllPermissions } = useAuth();
  const [isAuthorizing, setIsAuthorizing] = useState(false);
  const [isAuthorized, setIsAuthorized] = useState<boolean | null>(null);

  useEffect(() => {
    // 認証チェック中またはエラー状態ならスキップ
    if (state.status === 'loading') {
      return;
    }

    // 未認証の場合
    if (!state.isAuthenticated) {
      if (redirectTo && navigate) {
        navigate(redirectTo);
      }
      return;
    }

    // 認可チェック
    const checkAuthorization = async () => {
      setIsAuthorizing(true);

      try {
        let authorized = true;

        // ロールチェック
        if (roles && roles.length > 0) {
          authorized = authorized && hasAnyRole(roles);
        }

        // パーミッションチェック
        if (permissions && permissions.length > 0) {
          authorized = authorized && hasAllPermissions(permissions);
        }

        // カスタム認可チェック
        if (authorize && state.user) {
          const customResult = await authorize(state.user);
          authorized = authorized && customResult;
        }

        setIsAuthorized(authorized);

        // 認可失敗時のリダイレクト
        if (!authorized && forbiddenRedirectTo && navigate) {
          navigate(forbiddenRedirectTo);
        }
      } catch {
        setIsAuthorized(false);
      } finally {
        setIsAuthorizing(false);
      }
    };

    checkAuthorization();
  }, [
    state.status,
    state.isAuthenticated,
    state.user,
    roles,
    permissions,
    authorize,
    redirectTo,
    forbiddenRedirectTo,
    navigate,
    hasAnyRole,
    hasAllPermissions,
  ]);

  // ローディング中
  if (state.status === 'loading' || isAuthorizing) {
    return <>{loadingComponent ?? null}</>;
  }

  // 未認証
  if (!state.isAuthenticated) {
    if (!redirectTo) {
      return <>{unauthenticatedComponent ?? null}</>;
    }
    return null;
  }

  // 認可チェックがまだ完了していない
  if (isAuthorized === null) {
    return <>{loadingComponent ?? null}</>;
  }

  // 権限不足
  if (!isAuthorized) {
    if (!forbiddenRedirectTo) {
      return <>{forbiddenComponent ?? null}</>;
    }
    return null;
  }

  // 認証・認可OK
  return <>{children}</>;
}

/**
 * 認証が必要なルート用のラッパー
 */
export function RequireAuth({
  children,
  redirectTo = '/login',
  loadingComponent,
  unauthenticatedComponent,
  navigate,
}: Pick<
  AuthGuardProps,
  | 'children'
  | 'redirectTo'
  | 'loadingComponent'
  | 'unauthenticatedComponent'
  | 'navigate'
>) {
  return (
    <AuthGuard
      redirectTo={redirectTo}
      loadingComponent={loadingComponent}
      unauthenticatedComponent={unauthenticatedComponent}
      navigate={navigate}
    >
      {children}
    </AuthGuard>
  );
}

/**
 * 特定のロールが必要なルート用のラッパー
 */
export function RequireRole({
  children,
  role,
  roles: rolesProp,
  redirectTo = '/login',
  forbiddenRedirectTo = '/forbidden',
  loadingComponent,
  unauthenticatedComponent,
  forbiddenComponent,
  navigate,
}: Pick<
  AuthGuardProps,
  | 'children'
  | 'roles'
  | 'redirectTo'
  | 'forbiddenRedirectTo'
  | 'loadingComponent'
  | 'unauthenticatedComponent'
  | 'forbiddenComponent'
  | 'navigate'
> & { role?: string }) {
  const roles = rolesProp ?? (role ? [role] : undefined);

  return (
    <AuthGuard
      roles={roles}
      redirectTo={redirectTo}
      forbiddenRedirectTo={forbiddenRedirectTo}
      loadingComponent={loadingComponent}
      unauthenticatedComponent={unauthenticatedComponent}
      forbiddenComponent={forbiddenComponent}
      navigate={navigate}
    >
      {children}
    </AuthGuard>
  );
}

/**
 * 特定のパーミッションが必要なルート用のラッパー
 */
export function RequirePermission({
  children,
  permission,
  permissions: permissionsProp,
  redirectTo = '/login',
  forbiddenRedirectTo = '/forbidden',
  loadingComponent,
  unauthenticatedComponent,
  forbiddenComponent,
  navigate,
}: Pick<
  AuthGuardProps,
  | 'children'
  | 'permissions'
  | 'redirectTo'
  | 'forbiddenRedirectTo'
  | 'loadingComponent'
  | 'unauthenticatedComponent'
  | 'forbiddenComponent'
  | 'navigate'
> & { permission?: string }) {
  const permissions = permissionsProp ?? (permission ? [permission] : undefined);

  return (
    <AuthGuard
      permissions={permissions}
      redirectTo={redirectTo}
      forbiddenRedirectTo={forbiddenRedirectTo}
      loadingComponent={loadingComponent}
      unauthenticatedComponent={unauthenticatedComponent}
      forbiddenComponent={forbiddenComponent}
      navigate={navigate}
    >
      {children}
    </AuthGuard>
  );
}
