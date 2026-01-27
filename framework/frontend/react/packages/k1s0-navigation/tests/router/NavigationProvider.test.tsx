/**
 * NavigationProvider コンポーネントのテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import {
  NavigationProvider,
  useNavigationContext,
} from '../../src/router/NavigationProvider';
import type { NavigationConfig, ScreenDefinition } from '../../src/schema/types';

// テスト用の画面コンポーネント
const TestScreen = () => <div>Test Screen</div>;
const HomeScreen = () => <div>Home</div>;
const AdminScreen = () => <div>Admin</div>;

// テスト用の設定
const createTestConfig = (): NavigationConfig => ({
  version: 1,
  routes: [
    { path: '/', screen_id: 'home', title: 'Home' },
    { path: '/admin', screen_id: 'admin', title: 'Admin', requires: { permissions: ['admin'] } },
    { path: '/feature', screen_id: 'feature', title: 'Feature', requires: { flags: ['new_feature'] } },
    { path: '/redirect', redirect_to: '/home' },
  ],
  menu: [
    {
      id: 'main',
      label: 'Main Menu',
      items: [
        { label: 'Home', to: '/' },
        { label: 'Admin', to: '/admin', requires: { permissions: ['admin'] } },
      ],
    },
  ],
});

const createTestScreens = (): ScreenDefinition[] => [
  { id: 'home', component: HomeScreen },
  { id: 'admin', component: AdminScreen },
  { id: 'feature', component: TestScreen },
];

// テスト用ラッパー
const createWrapper = (
  config: NavigationConfig,
  screens: ScreenDefinition[],
  auth?: { permissions: string[]; flags: string[] },
  throwOnValidationError = false
) => {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <MemoryRouter>
        <NavigationProvider
          config={config}
          screens={screens}
          auth={auth}
          throwOnValidationError={throwOnValidationError}
        >
          {children}
        </NavigationProvider>
      </MemoryRouter>
    );
  };
};

describe('NavigationProvider', () => {
  describe('基本機能', () => {
    it('設定が正しくコンテキストに反映されること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens),
      });

      expect(result.current.config).toBe(config);
      expect(result.current.isValid).toBe(true);
      expect(result.current.errors).toHaveLength(0);
    });

    it('画面レジストリが正しく構築されること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens),
      });

      expect(result.current.screens.size).toBe(3);
      expect(result.current.screens.get('home')).toBeDefined();
      expect(result.current.screens.get('admin')).toBeDefined();
      expect(result.current.screens.get('feature')).toBeDefined();
    });

    it('NavigationProvider 外で useNavigationContext を使用するとエラーになること', () => {
      expect(() => {
        renderHook(() => useNavigationContext());
      }).toThrow('useNavigationContext は NavigationProvider 内で使用してください');
    });
  });

  describe('checkRequires 関数', () => {
    it('requires が undefined の場合は true を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens),
      });

      expect(result.current.checkRequires(undefined)).toBe(true);
    });

    it('権限がない場合は false を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: [], flags: [] }),
      });

      expect(result.current.checkRequires({ permissions: ['admin'] })).toBe(false);
    });

    it('権限がある場合は true を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: ['admin'], flags: [] }),
      });

      expect(result.current.checkRequires({ permissions: ['admin'] })).toBe(true);
    });

    it('フラグがない場合は false を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: [], flags: [] }),
      });

      expect(result.current.checkRequires({ flags: ['new_feature'] })).toBe(false);
    });

    it('フラグがある場合は true を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: [], flags: ['new_feature'] }),
      });

      expect(result.current.checkRequires({ flags: ['new_feature'] })).toBe(true);
    });

    it('複数の権限が全て必要な場合、全て満たさないと false を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: ['admin'], flags: [] }),
      });

      expect(result.current.checkRequires({ permissions: ['admin', 'super'] })).toBe(false);
    });

    it('複数の権限が全て満たされる場合は true を返すこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, { permissions: ['admin', 'super'], flags: [] }),
      });

      expect(result.current.checkRequires({ permissions: ['admin', 'super'] })).toBe(true);
    });
  });

  describe('バリデーション', () => {
    it('無効な設定でバリデーションエラーが発生すること', () => {
      const invalidConfig: NavigationConfig = {
        version: 1,
        routes: [
          { path: '', screen_id: 'home' }, // 無効なパス
        ],
        menu: [],
      };
      const screens = createTestScreens();

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(invalidConfig, screens, undefined, false),
      });

      expect(result.current.isValid).toBe(false);
      expect(result.current.errors.length).toBeGreaterThan(0);
    });

    it('存在しない screen_id で整合性エラーが発生すること', () => {
      const config: NavigationConfig = {
        version: 1,
        routes: [
          { path: '/', screen_id: 'nonexistent' },
        ],
        menu: [],
      };
      const screens: ScreenDefinition[] = [];

      const { result } = renderHook(() => useNavigationContext(), {
        wrapper: createWrapper(config, screens, undefined, false),
      });

      expect(result.current.isValid).toBe(false);
    });

    it('onValidationError が呼ばれること', () => {
      const onValidationError = vi.fn();
      const invalidConfig: NavigationConfig = {
        version: 1,
        routes: [
          { path: '/', screen_id: 'nonexistent' },
        ],
        menu: [],
      };

      const Wrapper = ({ children }: { children: ReactNode }) => (
        <MemoryRouter>
          <NavigationProvider
            config={invalidConfig}
            screens={[]}
            onValidationError={onValidationError}
            throwOnValidationError={false}
          >
            {children}
          </NavigationProvider>
        </MemoryRouter>
      );

      renderHook(() => useNavigationContext(), { wrapper: Wrapper });

      expect(onValidationError).toHaveBeenCalled();
    });
  });
});
