/**
 * PermissionGuard コンポーネントのテスト
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { renderHook } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { NavigationProvider } from '../../src/router/NavigationProvider';
import {
  PermissionGuard,
  useHasPermission,
  useHasAllPermissions,
  useHasAnyPermission,
} from '../../src/guards/PermissionGuard';
import type { NavigationConfig, ScreenDefinition } from '../../src/schema/types';

// テスト用の画面コンポーネント
const TestScreen = () => <div>Test Screen</div>;

// テスト用の設定
const createTestConfig = (): NavigationConfig => ({
  version: 1,
  routes: [
    { path: '/', screen_id: 'home', title: 'Home' },
  ],
  menu: [],
});

const createTestScreens = (): ScreenDefinition[] => [
  { id: 'home', component: TestScreen },
];

// テスト用ラッパー
const createWrapper = (permissions: string[] = [], flags: string[] = []) => {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <MemoryRouter>
        <NavigationProvider
          config={createTestConfig()}
          screens={createTestScreens()}
          auth={{ permissions, flags }}
          throwOnValidationError={false}
        >
          {children}
        </NavigationProvider>
      </MemoryRouter>
    );
  };
};

describe('PermissionGuard', () => {
  describe('基本的な動作', () => {
    it('必要な権限がある場合、子要素が表示されること', () => {
      render(
        <PermissionGuard permissions={['read']}>
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['read', 'write']) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });

    it('必要な権限がない場合、子要素が表示されないこと', () => {
      render(
        <PermissionGuard permissions={['admin']}>
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['read']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
    });

    it('権限がない場合、fallback が表示されること', () => {
      render(
        <PermissionGuard
          permissions={['admin']}
          fallback={<div data-testid="fallback">Access Denied</div>}
        >
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['read']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      expect(screen.getByTestId('fallback')).toBeInTheDocument();
    });
  });

  describe('複数権限', () => {
    it('複数権限が全て必要な場合、全て満たさないと表示されないこと', () => {
      render(
        <PermissionGuard permissions={['read', 'write']}>
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['read']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
    });

    it('複数権限が全て満たされる場合、表示されること', () => {
      render(
        <PermissionGuard permissions={['read', 'write']}>
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['read', 'write', 'delete']) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });
  });

  describe('空の権限配列', () => {
    it('空の権限配列の場合、子要素が表示されること', () => {
      render(
        <PermissionGuard permissions={[]}>
          <div data-testid="protected-content">Protected Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper([]) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });
  });
});

describe('useHasPermission', () => {
  it('権限がある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasPermission('read'), {
      wrapper: createWrapper(['read', 'write']),
    });

    expect(result.current).toBe(true);
  });

  it('権限がない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasPermission('admin'), {
      wrapper: createWrapper(['read', 'write']),
    });

    expect(result.current).toBe(false);
  });
});

describe('useHasAllPermissions', () => {
  it('全ての権限がある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAllPermissions(['read', 'write']), {
      wrapper: createWrapper(['read', 'write', 'delete']),
    });

    expect(result.current).toBe(true);
  });

  it('一部の権限がない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAllPermissions(['read', 'admin']), {
      wrapper: createWrapper(['read', 'write']),
    });

    expect(result.current).toBe(false);
  });

  it('空の配列の場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAllPermissions([]), {
      wrapper: createWrapper(['read']),
    });

    expect(result.current).toBe(true);
  });
});

describe('useHasAnyPermission', () => {
  it('いずれかの権限がある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAnyPermission(['admin', 'read']), {
      wrapper: createWrapper(['read']),
    });

    expect(result.current).toBe(true);
  });

  it('全ての権限がない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAnyPermission(['admin', 'super']), {
      wrapper: createWrapper(['read', 'write']),
    });

    expect(result.current).toBe(false);
  });

  it('空の配列の場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAnyPermission([]), {
      wrapper: createWrapper(['read']),
    });

    expect(result.current).toBe(false);
  });
});
