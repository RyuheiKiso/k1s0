/**
 * useNavigation - ナビゲーション操作用フック
 */

import { useCallback, useMemo } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useNavigationContext } from './NavigationProvider';
import type { RouteConfig, MenuGroupConfig, MenuItemConfig } from '../schema/types';

/** ナビゲーションフックの戻り値 */
export interface UseNavigationResult {
  /** 現在のパス */
  currentPath: string;
  /** 現在のルート設定 */
  currentRoute: RouteConfig | undefined;
  /** 指定パスに遷移 */
  navigateTo: (path: string) => void;
  /** 権限を満たすルート一覧 */
  accessibleRoutes: RouteConfig[];
  /** 権限を満たすメニュー一覧 */
  accessibleMenus: MenuGroupConfig[];
  /** 指定パスにアクセス可能か */
  canAccess: (path: string) => boolean;
}

/**
 * useNavigation フック
 *
 * ナビゲーション操作と現在の状態を提供する。
 */
export function useNavigation(): UseNavigationResult {
  const navigate = useNavigate();
  const location = useLocation();
  const { config, checkRequires } = useNavigationContext();

  const currentPath = location.pathname;

  // 現在のルート設定を取得
  const currentRoute = useMemo(() => {
    return config.routes.find((route) => {
      // 動的パラメータを含むパスのマッチング（簡易版）
      const routePattern = route.path.replace(/:[^/]+/g, '[^/]+');
      const regex = new RegExp(`^${routePattern}$`);
      return regex.test(currentPath);
    });
  }, [config.routes, currentPath]);

  // 遷移関数
  const navigateTo = useCallback(
    (path: string) => {
      navigate(path);
    },
    [navigate]
  );

  // アクセス可能なルート一覧
  const accessibleRoutes = useMemo(() => {
    return config.routes.filter(
      (route) => !route.redirect_to && checkRequires(route.requires)
    );
  }, [config.routes, checkRequires]);

  // アクセス可能なメニュー一覧
  const accessibleMenus = useMemo(() => {
    return config.menu
      .map((group): MenuGroupConfig => ({
        ...group,
        items: group.items.filter((item) => checkRequires(item.requires)),
      }))
      .filter((group) => group.items.length > 0);
  }, [config.menu, checkRequires]);

  // 指定パスにアクセス可能か判定
  const canAccess = useCallback(
    (path: string): boolean => {
      const route = config.routes.find((r) => r.path === path);
      if (!route) return false;
      return checkRequires(route.requires);
    },
    [config.routes, checkRequires]
  );

  return {
    currentPath,
    currentRoute,
    navigateTo,
    accessibleRoutes,
    accessibleMenus,
    canAccess,
  };
}

/** メニュー項目がアクティブか判定するユーティリティ */
export function isMenuItemActive(
  item: MenuItemConfig,
  currentPath: string
): boolean {
  // 完全一致
  if (item.to === currentPath) return true;

  // 子パスの場合もアクティブとする（例: /users/123 → /users がアクティブ）
  if (item.to !== '/' && currentPath.startsWith(item.to + '/')) {
    return true;
  }

  return false;
}
