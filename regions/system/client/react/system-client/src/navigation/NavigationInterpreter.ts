import type React from 'react';
import { parse as parseYaml } from 'yaml';
import type {
  NavigationResponse,
  NavigationRoute,
  NavigationGuard,
  ComponentRegistry,
} from './types';

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
    return res.json() as Promise<NavigationResponse>;
  }

  private parseNavigation(text: string): NavigationResponse {
    const trimmed = text.trim();
    if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
      return JSON.parse(trimmed) as NavigationResponse;
    }

    return parseYaml(trimmed) as NavigationResponse;
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
