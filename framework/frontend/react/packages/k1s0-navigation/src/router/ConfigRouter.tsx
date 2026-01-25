/**
 * ConfigRouter - 設定からルートを生成するコンポーネント
 */

import { Routes, Route, Navigate } from 'react-router-dom';
import { useNavigationContext } from './NavigationProvider';
import type { RouteConfig } from '../schema/types';

/** ルートガードのプロパティ */
interface RouteGuardProps {
  route: RouteConfig;
  children: React.ReactNode;
}

/**
 * RouteGuard - 権限・フラグによるアクセス制御
 */
function RouteGuard({ route, children }: RouteGuardProps) {
  const { checkRequires } = useNavigationContext();

  if (!checkRequires(route.requires)) {
    // 権限がない場合はホームにリダイレクト
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}

/** ルート要素を生成 */
function RouteElement({ route }: { route: RouteConfig }) {
  const { screens } = useNavigationContext();

  // リダイレクトの場合
  if (route.redirect_to) {
    return <Navigate to={route.redirect_to} replace />;
  }

  // 画面IDから画面コンポーネントを取得
  if (route.screen_id) {
    const screen = screens.get(route.screen_id);
    if (!screen) {
      // 開発時のエラー表示（本番では validateConfigIntegrity で検知済み）
      return (
        <div style={{ padding: 20, color: 'red' }}>
          Error: Screen "{route.screen_id}" not found
        </div>
      );
    }

    const ScreenComponent = screen.component;
    return (
      <RouteGuard route={route}>
        <ScreenComponent />
      </RouteGuard>
    );
  }

  return null;
}

/**
 * ConfigRouter コンポーネント
 *
 * NavigationConfig の routes から React Router の Routes を生成する。
 */
export function ConfigRouter() {
  const { config, isValid } = useNavigationContext();

  if (!isValid) {
    return (
      <div style={{ padding: 20, color: 'red' }}>
        Navigation configuration is invalid. Check console for details.
      </div>
    );
  }

  return (
    <Routes>
      {config.routes.map((route) => (
        <Route
          key={route.path}
          path={route.path}
          element={<RouteElement route={route} />}
        />
      ))}
      {/* 404 フォールバック */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
