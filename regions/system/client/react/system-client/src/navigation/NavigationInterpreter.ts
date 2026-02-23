import type React from 'react';
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
  component?: React.ComponentType<any>;
  lazyComponent?: () => Promise<{ default: React.ComponentType<any> }>;
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
    // JSON format supported; YAML requires js-yaml (not in dependencies)
    return JSON.parse(text) as NavigationResponse;
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

        let component: React.ComponentType<any> | undefined;
        let lazyComponent: (() => Promise<{ default: React.ComponentType<any> }>) | undefined;

        if (typeof entry === 'function' && isLazyImport(entry)) {
          lazyComponent = entry as () => Promise<{ default: React.ComponentType<any> }>;
        } else if (entry) {
          component = entry as React.ComponentType<any>;
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
  entry: React.ComponentType<any> | (() => Promise<{ default: React.ComponentType<any> }>),
): entry is () => Promise<{ default: React.ComponentType<any> }> {
  // Class components have prototype.isReactComponent
  if (typeof entry === 'function' && entry.prototype?.isReactComponent) {
    return false;
  }
  // React.memo / forwardRef have $$typeof
  if ((entry as any).$$typeof) {
    return false;
  }
  // Lazy imports are zero-arity arrow functions without a prototype property
  // Function components (including arrow) always accept at least props
  // Lazy: () => import('...') has length 0
  // Component: (props) => <div/> or function Comp(props) has length >= 0
  // Best heuristic: lazy imports have no 'prototype' (arrow fn) and length === 0
  const fn = entry as Function;
  return !fn.prototype && fn.length === 0;
}
