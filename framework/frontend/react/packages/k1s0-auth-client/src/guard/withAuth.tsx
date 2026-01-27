import React, { type ComponentType, type ReactNode } from 'react';
import { AuthGuard } from './AuthGuard.js';
import type { AuthGuardConfig } from '../types.js';

/**
 * withAuth HOC のオプション
 */
interface WithAuthOptions extends AuthGuardConfig {
  /** 認証チェック中に表示するコンポーネント */
  LoadingComponent?: ComponentType;
  /** 認証されていない場合に表示するコンポーネント */
  UnauthenticatedComponent?: ComponentType;
  /** 権限不足の場合に表示するコンポーネント */
  ForbiddenComponent?: ComponentType;
  /** リダイレクト関数 */
  navigate?: (to: string) => void;
}

/**
 * 認証が必要なコンポーネント用の HOC
 *
 * @example
 * ```tsx
 * const ProtectedComponent = withAuth(MyComponent, {
 *   redirectTo: '/login',
 *   roles: ['admin'],
 * });
 * ```
 */
export function withAuth<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: WithAuthOptions = {}
): ComponentType<P> {
  const {
    LoadingComponent,
    UnauthenticatedComponent,
    ForbiddenComponent,
    navigate,
    ...guardConfig
  } = options;

  function WithAuthComponent(props: P) {
    return (
      <AuthGuard
        {...guardConfig}
        loadingComponent={LoadingComponent ? <LoadingComponent /> : undefined}
        unauthenticatedComponent={
          UnauthenticatedComponent ? <UnauthenticatedComponent /> : undefined
        }
        forbiddenComponent={
          ForbiddenComponent ? <ForbiddenComponent /> : undefined
        }
        navigate={navigate}
      >
        <WrappedComponent {...props} />
      </AuthGuard>
    );
  }

  WithAuthComponent.displayName = `withAuth(${
    WrappedComponent.displayName || WrappedComponent.name || 'Component'
  })`;

  return WithAuthComponent;
}

/**
 * 認証が必要なコンポーネント用の HOC（リダイレクト付き）
 */
export function withRequireAuth<P extends object>(
  WrappedComponent: ComponentType<P>,
  redirectTo: string = '/login',
  navigate?: (to: string) => void
): ComponentType<P> {
  return withAuth(WrappedComponent, { redirectTo, navigate });
}

/**
 * 特定のロールが必要なコンポーネント用の HOC
 */
export function withRequireRole<P extends object>(
  WrappedComponent: ComponentType<P>,
  roles: string[],
  options: Omit<WithAuthOptions, 'roles'> = {}
): ComponentType<P> {
  return withAuth(WrappedComponent, { ...options, roles });
}

/**
 * 特定のパーミッションが必要なコンポーネント用の HOC
 */
export function withRequirePermission<P extends object>(
  WrappedComponent: ComponentType<P>,
  permissions: string[],
  options: Omit<WithAuthOptions, 'permissions'> = {}
): ComponentType<P> {
  return withAuth(WrappedComponent, { ...options, permissions });
}
