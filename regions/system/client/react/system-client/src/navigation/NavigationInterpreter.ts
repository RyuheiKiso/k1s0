import type React from 'react';
import { parse as parseYaml } from 'yaml';
import { z } from 'zod';
import type {
  NavigationResponse,
  NavigationRoute,
  NavigationGuard,
  ComponentRegistry,
} from './types';

// HIGH-FE-003 対応: NavigationResponse の zod スキーマ定義。
// サーバーから受け取ったナビゲーション設定の実行時型検証を行い、
// 不正なスキーマによるランタイムエラーを早期に検出する。
const NavigationGuardSchema = z.object({
  id: z.string(),
  type: z.enum(['auth_required', 'role_required', 'redirect_if_authenticated']),
  redirect_to: z.string(),
  roles: z.array(z.string()).optional(),
});

// NavigationRoute は再帰的な構造を持つため lazy() で定義する
const NavigationRouteSchema: z.ZodType<NavigationRoute> = z.lazy(() =>
  z.object({
    id: z.string(),
    path: z.string(),
    component_id: z.string().optional(),
    guards: z.array(z.string()).optional(),
    transition: z.enum(['fade', 'slide', 'modal']).optional(),
    redirect_to: z.string().optional(),
    children: z.array(NavigationRouteSchema).optional(),
    params: z.array(
      z.object({
        name: z.string(),
        type: z.enum(['string', 'int', 'uuid']),
      })
    ).optional(),
  })
);

// NavigationResponse のトップレベルスキーマ
const NavigationResponseSchema = z.object({
  routes: z.array(NavigationRouteSchema),
  guards: z.array(NavigationGuardSchema),
});

export interface NavigationConfig {
  mode: 'remote' | 'local';
  remoteUrl?: string;
  localConfigPath?: string;
  componentRegistry: ComponentRegistry;
}

export interface ResolvedRoute {
  id: string;
  path: string;
  component?: React.ComponentType<Record<string, unknown>>;
  lazyComponent?: () => Promise<{ default: React.ComponentType<Record<string, unknown>> }>;
  guards: NavigationGuard[];
  transition?: string;
  redirect_to?: string;
  children: ResolvedRoute[];
}

export interface RouterResult {
  routes: ResolvedRoute[];
  guards: NavigationGuard[];
  raw: NavigationResponse;
}

export class NavigationInterpreter {
  constructor(private readonly config: NavigationConfig) {}

  async build(): Promise<RouterResult> {
    const nav = await this.fetchNavigation();
    return this.buildRouter(nav);
  }

  private async fetchNavigation(): Promise<NavigationResponse> {
    if (this.config.mode === 'local') {
      const res = await fetch(this.config.localConfigPath ?? '/navigation.yaml');
      const text = await res.text();
      return this.parseNavigation(text);
    }
    const res = await fetch(this.config.remoteUrl ?? '/api/v1/navigation');
    // HIGH-FE-003 対応: サーバーからの JSON レスポンスを zod で実行時検証する。
    // 不正なスキーマが返された場合は ZodError をスローして呼び出し元に通知する。
    return NavigationResponseSchema.parse(await res.json());
  }

  private parseNavigation(text: string): NavigationResponse {
    const trimmed = text.trim();
    if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
      // HIGH-FE-003 対応: JSON パース後に zod で実行時検証する
      return NavigationResponseSchema.parse(JSON.parse(trimmed));
    }

    // HIGH-FE-003 対応: YAML パース後に zod で実行時検証する
    return NavigationResponseSchema.parse(parseYaml(trimmed));
  }

  private buildRouter(nav: NavigationResponse): RouterResult {
    const guardMap = new Map<string, NavigationGuard>();
    for (const guard of nav.guards) {
      guardMap.set(guard.id, guard);
    }

    const resolveRoutes = (routes: NavigationRoute[]): ResolvedRoute[] =>
      routes.map((route) => {
        const guards = (route.guards ?? [])
          .map((id) => guardMap.get(id))
          .filter((g): g is NavigationGuard => g !== undefined);

        const entry = route.component_id
          ? this.config.componentRegistry[route.component_id]
          : undefined;

        let component: React.ComponentType<Record<string, unknown>> | undefined;
        let lazyComponent: (() => Promise<{ default: React.ComponentType<Record<string, unknown>> }>) | undefined;

        if (typeof entry === 'function' && isLazyImport(entry)) {
          lazyComponent = entry as () => Promise<{ default: React.ComponentType<Record<string, unknown>> }>;
        } else if (entry) {
          component = entry as React.ComponentType<Record<string, unknown>>;
        }

        return {
          id: route.id,
          path: route.path,
          component,
          lazyComponent,
          guards,
          transition: route.transition,
          redirect_to: route.redirect_to,
          children: route.children ? resolveRoutes(route.children) : [],
        };
      });

    return {
      routes: resolveRoutes(nav.routes),
      guards: nav.guards,
      raw: nav,
    };
  }
}

function isLazyImport(
  entry: React.ComponentType<Record<string, unknown>> | (() => Promise<{ default: React.ComponentType<Record<string, unknown>> }>),
): entry is () => Promise<{ default: React.ComponentType<Record<string, unknown>> }> {
  // Class components have prototype.isReactComponent
  if (typeof entry === 'function' && entry.prototype?.isReactComponent) {
    return false;
  }
  // React.memo / forwardRef は $$typeof プロパティを持つ
  if ((entry as unknown as Record<string, unknown>).$$typeof) {
    return false;
  }
  // 遅延インポートはprototypeを持たないゼロ引数のアロー関数
  // コンポーネント関数は常にpropsを受け取る（length >= 0）
  // 遅延インポート: () => import('...') は length === 0
  // ヒューリスティック: prototypeを持たず length === 0 なら遅延インポートと判定
  const fn = entry as (...args: unknown[]) => unknown;
  return !fn.prototype && fn.length === 0;
}
