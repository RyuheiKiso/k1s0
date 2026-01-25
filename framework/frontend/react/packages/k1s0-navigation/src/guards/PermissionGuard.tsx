/**
 * PermissionGuard - 権限による表示制御
 */

import type { ReactNode } from 'react';
import { useNavigationContext } from '../router/NavigationProvider';

/** PermissionGuard のプロパティ */
export interface PermissionGuardProps {
  /** 必要な権限（すべて満たす必要がある） */
  permissions: string[];
  /** 子要素 */
  children: ReactNode;
  /** 権限がない場合のフォールバック */
  fallback?: ReactNode;
}

/**
 * PermissionGuard コンポーネント
 *
 * 指定した権限をすべて持つ場合のみ子要素を表示する。
 */
export function PermissionGuard({
  permissions,
  children,
  fallback = null,
}: PermissionGuardProps) {
  const { auth } = useNavigationContext();

  const hasAllPermissions = permissions.every((p) =>
    auth.permissions.includes(p)
  );

  if (!hasAllPermissions) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}

/** 権限があるか判定するフック */
export function useHasPermission(permission: string): boolean {
  const { auth } = useNavigationContext();
  return auth.permissions.includes(permission);
}

/** 複数権限をすべて持つか判定するフック */
export function useHasAllPermissions(permissions: string[]): boolean {
  const { auth } = useNavigationContext();
  return permissions.every((p) => auth.permissions.includes(p));
}

/** 複数権限のいずれかを持つか判定するフック */
export function useHasAnyPermission(permissions: string[]): boolean {
  const { auth } = useNavigationContext();
  return permissions.some((p) => auth.permissions.includes(p));
}
