import { useState } from 'react';
import type { RouterResult, ResolvedRoute } from '../NavigationInterpreter';
import styles from './NavigationDevTools.module.css';

// NavigationDevToolsのProps定義
interface NavigationDevToolsProps {
  router: RouterResult;
  currentPath?: string;
}

// ナビゲーションDevToolsコンポーネント: 開発環境のみで表示されるルート情報パネル
export function NavigationDevTools({ router, currentPath }: NavigationDevToolsProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // 本番環境では非表示
  if (import.meta.env.PROD) {
    return null;
  }

  // ルートツリーをフラット化
  const flatRoutes = flattenRoutes(router.routes);
  // 現在のパスに対応するアクティブルートを検索
  const activeRoute = currentPath
    ? flatRoutes.find((r) => r.path === currentPath)
    : undefined;

  return (
    <div
      className={`${styles.container} ${isExpanded ? styles.expanded : ''}`}
      role="complementary"
      aria-label="ナビゲーション開発ツール"
    >
      <div
        className={`${styles.toggle} ${isExpanded ? styles.toggleExpanded : ''}`}
        onClick={() => setIsExpanded(!isExpanded)}
        role="button"
        tabIndex={0}
        aria-expanded={isExpanded}
        aria-label="DevToolsを切り替え"
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') setIsExpanded(!isExpanded);
        }}
      >
        Nav DevTools {isExpanded ? '[-]' : '[+]'}
      </div>
      {isExpanded && (
        <>
          {/* アクティブルート情報 */}
          {activeRoute && (
            <div className={styles.activeRoute}>
              <div className={styles.activeRouteId}>Active: {activeRoute.id}</div>
              <div>Path: {activeRoute.path}</div>
              {activeRoute.guards.length > 0 && (
                <div>Guards: {activeRoute.guards.map((g) => g.id).join(', ')}</div>
              )}
            </div>
          )}
          {/* ルート・ガード概要 */}
          <div className={styles.summary}>
            Routes ({flatRoutes.length}) | Guards ({router.guards.length})
          </div>
          {/* 全ルートの一覧 */}
          {flatRoutes.map((route) => (
            <div
              key={route.id}
              className={`${styles.routeRow} ${route.id === activeRoute?.id ? styles.routeRowActive : ''}`}
            >
              <span className={styles.routeId}>{route.id}</span>{' '}
              <span className={styles.routePath}>{route.path}</span>
              {route.guards.length > 0 && (
                <span className={styles.guards}>
                  [{route.guards.map((g) => g.type).join(',')}]
                </span>
              )}
            </div>
          ))}
        </>
      )}
    </div>
  );
}

// ルートツリーをフラットな配列に変換する再帰関数
function flattenRoutes(routes: ResolvedRoute[]): ResolvedRoute[] {
  const result: ResolvedRoute[] = [];
  for (const route of routes) {
    result.push(route);
    if (route.children.length > 0) {
      result.push(...flattenRoutes(route.children));
    }
  }
  return result;
}
