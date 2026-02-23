import { useState } from 'react';
import type { RouterResult, ResolvedRoute } from '../NavigationInterpreter';

interface NavigationDevToolsProps {
  router: RouterResult;
  currentPath?: string;
}

export function NavigationDevTools({ router, currentPath }: NavigationDevToolsProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (process.env.NODE_ENV === 'production') {
    return null;
  }

  const flatRoutes = flattenRoutes(router.routes);
  const activeRoute = currentPath
    ? flatRoutes.find((r) => r.path === currentPath)
    : undefined;

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 16,
        right: 16,
        background: '#1a1a2e',
        color: '#e0e0e0',
        borderRadius: 8,
        padding: 12,
        fontSize: 12,
        fontFamily: 'monospace',
        zIndex: 99999,
        maxWidth: 360,
        maxHeight: isExpanded ? 400 : 'auto',
        overflow: 'auto',
        boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
      }}
    >
      <div
        style={{ cursor: 'pointer', marginBottom: isExpanded ? 8 : 0, fontWeight: 'bold' }}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        Nav DevTools {isExpanded ? '[-]' : '[+]'}
      </div>
      {isExpanded && (
        <>
          {activeRoute && (
            <div style={{ marginBottom: 8, padding: 4, background: '#16213e', borderRadius: 4 }}>
              <div style={{ color: '#0ff' }}>Active: {activeRoute.id}</div>
              <div>Path: {activeRoute.path}</div>
              {activeRoute.guards.length > 0 && (
                <div>Guards: {activeRoute.guards.map((g) => g.id).join(', ')}</div>
              )}
            </div>
          )}
          <div style={{ color: '#888', marginBottom: 4 }}>
            Routes ({flatRoutes.length}) | Guards ({router.guards.length})
          </div>
          {flatRoutes.map((route) => (
            <div
              key={route.id}
              style={{
                padding: '2px 4px',
                background: route.id === activeRoute?.id ? '#16213e' : 'transparent',
                borderRadius: 2,
              }}
            >
              <span style={{ color: '#7b68ee' }}>{route.id}</span>{' '}
              <span style={{ color: '#666' }}>{route.path}</span>
              {route.guards.length > 0 && (
                <span style={{ color: '#ff6b6b', marginLeft: 4 }}>
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
