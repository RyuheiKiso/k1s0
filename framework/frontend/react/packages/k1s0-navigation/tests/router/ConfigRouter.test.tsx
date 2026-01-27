/**
 * ConfigRouter コンポーネントのテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { NavigationProvider } from '../../src/router/NavigationProvider';
import { ConfigRouter } from '../../src/router/ConfigRouter';
import type { NavigationConfig, ScreenDefinition } from '../../src/schema/types';

// テスト用の画面コンポーネント
const HomeScreen = () => <div data-testid="home-screen">Home Screen</div>;
const AdminScreen = () => <div data-testid="admin-screen">Admin Screen</div>;
const ProtectedScreen = () => <div data-testid="protected-screen">Protected Screen</div>;

const createTestConfig = (): NavigationConfig => ({
  version: 1,
  routes: [
    { path: '/', screen_id: 'home', title: 'Home' },
    { path: '/admin', screen_id: 'admin', title: 'Admin', requires: { permissions: ['admin'] } },
    { path: '/protected', screen_id: 'protected', title: 'Protected', requires: { flags: ['beta'] } },
    { path: '/old', redirect_to: '/' },
  ],
  menu: [],
});

const createTestScreens = (): ScreenDefinition[] => [
  { id: 'home', component: HomeScreen },
  { id: 'admin', component: AdminScreen },
  { id: 'protected', component: ProtectedScreen },
];

interface TestWrapperProps {
  initialEntries?: string[];
  auth?: { permissions: string[]; flags: string[] };
  config?: NavigationConfig;
  screens?: ScreenDefinition[];
}

const TestWrapper = ({
  initialEntries = ['/'],
  auth = { permissions: [], flags: [] },
  config = createTestConfig(),
  screens = createTestScreens(),
}: TestWrapperProps) => {
  return (
    <MemoryRouter initialEntries={initialEntries}>
      <NavigationProvider
        config={config}
        screens={screens}
        auth={auth}
        throwOnValidationError={false}
      >
        <ConfigRouter />
      </NavigationProvider>
    </MemoryRouter>
  );
};

describe('ConfigRouter', () => {
  describe('基本的なルーティング', () => {
    it('ホーム画面が表示されること', () => {
      render(<TestWrapper initialEntries={['/']} />);
      expect(screen.getByTestId('home-screen')).toBeInTheDocument();
    });

    it('管理者権限がある場合、管理画面が表示されること', () => {
      render(
        <TestWrapper
          initialEntries={['/admin']}
          auth={{ permissions: ['admin'], flags: [] }}
        />
      );
      expect(screen.getByTestId('admin-screen')).toBeInTheDocument();
    });

    it('フラグがある場合、保護された画面が表示されること', () => {
      render(
        <TestWrapper
          initialEntries={['/protected']}
          auth={{ permissions: [], flags: ['beta'] }}
        />
      );
      expect(screen.getByTestId('protected-screen')).toBeInTheDocument();
    });
  });

  describe('権限によるアクセス制御', () => {
    it('権限がない場合、ホームにリダイレクトされること', () => {
      render(
        <TestWrapper
          initialEntries={['/admin']}
          auth={{ permissions: [], flags: [] }}
        />
      );
      // リダイレクト後、ホーム画面が表示される
      expect(screen.getByTestId('home-screen')).toBeInTheDocument();
      expect(screen.queryByTestId('admin-screen')).not.toBeInTheDocument();
    });

    it('フラグがない場合、ホームにリダイレクトされること', () => {
      render(
        <TestWrapper
          initialEntries={['/protected']}
          auth={{ permissions: [], flags: [] }}
        />
      );
      expect(screen.getByTestId('home-screen')).toBeInTheDocument();
      expect(screen.queryByTestId('protected-screen')).not.toBeInTheDocument();
    });
  });

  describe('リダイレクト', () => {
    it('リダイレクト設定が正しく動作すること', () => {
      render(
        <TestWrapper
          initialEntries={['/old']}
          auth={{ permissions: [], flags: [] }}
        />
      );
      expect(screen.getByTestId('home-screen')).toBeInTheDocument();
    });
  });

  describe('404 フォールバック', () => {
    it('存在しないパスはホームにリダイレクトされること', () => {
      render(
        <TestWrapper
          initialEntries={['/nonexistent']}
          auth={{ permissions: [], flags: [] }}
        />
      );
      expect(screen.getByTestId('home-screen')).toBeInTheDocument();
    });
  });

  describe('不正な設定', () => {
    it('無効な設定の場合、エラーメッセージが表示されること', () => {
      const invalidConfig: NavigationConfig = {
        version: 1,
        routes: [
          { path: '', screen_id: 'home' }, // 無効なパス
        ],
        menu: [],
      };

      render(
        <TestWrapper
          config={invalidConfig}
          screens={createTestScreens()}
        />
      );

      expect(screen.getByText(/Navigation configuration is invalid/)).toBeInTheDocument();
    });

    it('存在しない screen_id の場合、エラーメッセージが表示されること', () => {
      const config: NavigationConfig = {
        version: 1,
        routes: [
          { path: '/', screen_id: 'nonexistent' },
        ],
        menu: [],
      };

      render(
        <TestWrapper
          config={config}
          screens={[]}
        />
      );

      // 設定が無効と判定されるため、設定エラーが表示される
      expect(screen.getByText(/Navigation configuration is invalid/)).toBeInTheDocument();
    });
  });
});
