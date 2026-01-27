/**
 * MenuBuilder コンポーネントのテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { renderHook } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { MemoryRouter, useNavigate, useLocation } from 'react-router-dom';
import { NavigationProvider } from '../../src/router/NavigationProvider';
import { MenuBuilder, useMenuItems } from '../../src/menu/MenuBuilder';
import type { NavigationConfig, ScreenDefinition, MenuItemRenderProps, MenuGroupRenderProps } from '../../src/schema/types';

// react-router-dom のモック
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: vi.fn(),
    useLocation: vi.fn(),
  };
});

// テスト用の画面コンポーネント
const TestScreen = () => <div>Test Screen</div>;

// テスト用の設定
const createTestConfig = (): NavigationConfig => ({
  version: 1,
  routes: [
    { path: '/', screen_id: 'home', title: 'Home' },
    { path: '/users', screen_id: 'users', title: 'Users' },
    { path: '/admin', screen_id: 'admin', title: 'Admin', requires: { permissions: ['admin'] } },
    { path: '/settings', screen_id: 'settings', title: 'Settings' },
  ],
  menu: [
    {
      id: 'main',
      label: 'Main Menu',
      items: [
        { label: 'Home', to: '/', icon: 'home' },
        { label: 'Users', to: '/users', icon: 'people' },
        { label: 'Admin', to: '/admin', requires: { permissions: ['admin'] } },
      ],
    },
    {
      id: 'settings',
      label: 'Settings',
      items: [
        { label: 'Settings', to: '/settings', icon: 'settings' },
      ],
    },
  ],
});

const createTestScreens = (): ScreenDefinition[] => [
  { id: 'home', component: TestScreen },
  { id: 'users', component: TestScreen },
  { id: 'admin', component: TestScreen },
  { id: 'settings', component: TestScreen },
];

// テスト用ラッパー
const createWrapper = (
  config: NavigationConfig,
  screens: ScreenDefinition[],
  auth: { permissions: string[]; flags: string[] } = { permissions: [], flags: [] },
  initialEntries: string[] = ['/']
) => {
  const mockNavigate = vi.fn();
  (useNavigate as ReturnType<typeof vi.fn>).mockReturnValue(mockNavigate);
  (useLocation as ReturnType<typeof vi.fn>).mockReturnValue({ pathname: initialEntries[0] });

  return {
    wrapper: function Wrapper({ children }: { children: ReactNode }) {
      return (
        <MemoryRouter initialEntries={initialEntries}>
          <NavigationProvider
            config={config}
            screens={screens}
            auth={auth}
            throwOnValidationError={false}
          >
            {children}
          </NavigationProvider>
        </MemoryRouter>
      );
    },
    mockNavigate,
  };
};

describe('MenuBuilder', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('メニュー項目のレンダリング', () => {
    it('アクセス可能なメニュー項目が表示されること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens);

      const renderItem = ({ item, isActive, onClick }: MenuItemRenderProps) => (
        <button
          key={item.to}
          data-testid={`menu-item-${item.to}`}
          data-active={isActive}
          onClick={onClick}
        >
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} />,
        { wrapper }
      );

      expect(screen.getByTestId('menu-item-/')).toBeInTheDocument();
      expect(screen.getByTestId('menu-item-/users')).toBeInTheDocument();
      expect(screen.getByTestId('menu-item-/settings')).toBeInTheDocument();
    });

    it('権限がない場合、管理者メニューが表示されないこと', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens, { permissions: [], flags: [] });

      const renderItem = ({ item }: MenuItemRenderProps) => (
        <button key={item.to} data-testid={`menu-item-${item.to}`}>
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} />,
        { wrapper }
      );

      expect(screen.queryByTestId('menu-item-/admin')).not.toBeInTheDocument();
    });

    it('権限がある場合、管理者メニューが表示されること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens, { permissions: ['admin'], flags: [] });

      const renderItem = ({ item }: MenuItemRenderProps) => (
        <button key={item.to} data-testid={`menu-item-${item.to}`}>
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} />,
        { wrapper }
      );

      expect(screen.getByTestId('menu-item-/admin')).toBeInTheDocument();
    });
  });

  describe('アクティブ状態', () => {
    it('現在のパスと一致するメニュー項目がアクティブになること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens, { permissions: [], flags: [] }, ['/users']);

      const renderItem = ({ item, isActive }: MenuItemRenderProps) => (
        <button key={item.to} data-testid={`menu-item-${item.to}`} data-active={isActive}>
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} />,
        { wrapper }
      );

      expect(screen.getByTestId('menu-item-/users')).toHaveAttribute('data-active', 'true');
      expect(screen.getByTestId('menu-item-/')).toHaveAttribute('data-active', 'false');
    });
  });

  describe('グループフィルタ', () => {
    it('groupId を指定すると、そのグループのみ表示されること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens);

      const renderItem = ({ item }: MenuItemRenderProps) => (
        <button key={item.to} data-testid={`menu-item-${item.to}`}>
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} groupId="settings" />,
        { wrapper }
      );

      // settings グループのみ表示
      expect(screen.getByTestId('menu-item-/settings')).toBeInTheDocument();
      // main グループは非表示
      expect(screen.queryByTestId('menu-item-/')).not.toBeInTheDocument();
      expect(screen.queryByTestId('menu-item-/users')).not.toBeInTheDocument();
    });
  });

  describe('カスタムグループレンダリング', () => {
    it('renderGroup が正しく呼ばれること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper } = createWrapper(config, screens);

      const renderItem = ({ item }: MenuItemRenderProps) => (
        <button key={item.to}>{item.label}</button>
      );

      const renderGroup = ({ group, children }: MenuGroupRenderProps) => (
        <div key={group.id} data-testid={`group-${group.id}`}>
          <h3>{group.label}</h3>
          {children}
        </div>
      );

      render(
        <MenuBuilder renderItem={renderItem} renderGroup={renderGroup} />,
        { wrapper }
      );

      expect(screen.getByTestId('group-main')).toBeInTheDocument();
      expect(screen.getByTestId('group-settings')).toBeInTheDocument();
      expect(screen.getByText('Main Menu')).toBeInTheDocument();
      expect(screen.getByText('Settings')).toBeInTheDocument();
    });
  });

  describe('クリックハンドラ', () => {
    it('メニュー項目をクリックすると navigateTo が呼ばれること', () => {
      const config = createTestConfig();
      const screens = createTestScreens();
      const { wrapper, mockNavigate } = createWrapper(config, screens);

      const renderItem = ({ item, onClick }: MenuItemRenderProps) => (
        <button key={item.to} data-testid={`menu-item-${item.to}`} onClick={onClick}>
          {item.label}
        </button>
      );

      render(
        <MenuBuilder renderItem={renderItem} />,
        { wrapper }
      );

      fireEvent.click(screen.getByTestId('menu-item-/users'));
      expect(mockNavigate).toHaveBeenCalledWith('/users');
    });
  });
});

describe('useMenuItems', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('メニュー項目の配列を返すこと', () => {
    const config = createTestConfig();
    const screens = createTestScreens();
    const { wrapper } = createWrapper(config, screens);

    const { result } = renderHook(() => useMenuItems(), { wrapper });

    expect(result.current).toBeInstanceOf(Array);
    expect(result.current.length).toBeGreaterThan(0);
  });

  it('各項目に groupId と groupLabel が含まれること', () => {
    const config = createTestConfig();
    const screens = createTestScreens();
    const { wrapper } = createWrapper(config, screens);

    const { result } = renderHook(() => useMenuItems(), { wrapper });

    const mainItems = result.current.filter((item) => item.groupId === 'main');
    expect(mainItems.length).toBeGreaterThan(0);
    expect(mainItems[0].groupLabel).toBe('Main Menu');
  });

  it('groupId を指定すると、そのグループの項目のみ返すこと', () => {
    const config = createTestConfig();
    const screens = createTestScreens();
    const { wrapper } = createWrapper(config, screens);

    const { result } = renderHook(() => useMenuItems('settings'), { wrapper });

    expect(result.current.every((item) => item.groupId === 'settings')).toBe(true);
  });
});
