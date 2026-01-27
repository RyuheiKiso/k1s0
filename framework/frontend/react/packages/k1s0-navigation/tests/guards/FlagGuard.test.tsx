/**
 * FlagGuard コンポーネントのテスト
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { renderHook } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { NavigationProvider } from '../../src/router/NavigationProvider';
import {
  FlagGuard,
  useHasFlag,
  useHasAllFlags,
  useHasAnyFlag,
} from '../../src/guards/FlagGuard';
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
const createWrapper = (flags: string[] = [], permissions: string[] = []) => {
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

describe('FlagGuard', () => {
  describe('基本的な動作', () => {
    it('必要なフラグがある場合、子要素が表示されること', () => {
      render(
        <FlagGuard flags={['beta']}>
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper(['beta', 'experimental']) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });

    it('必要なフラグがない場合、子要素が表示されないこと', () => {
      render(
        <FlagGuard flags={['beta']}>
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper(['alpha']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
    });

    it('フラグがない場合、fallback が表示されること', () => {
      render(
        <FlagGuard
          flags={['beta']}
          fallback={<div data-testid="fallback">Feature not available</div>}
        >
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper(['alpha']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      expect(screen.getByTestId('fallback')).toBeInTheDocument();
    });
  });

  describe('複数フラグ', () => {
    it('複数フラグが全て必要な場合、全て満たさないと表示されないこと', () => {
      render(
        <FlagGuard flags={['beta', 'experimental']}>
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper(['beta']) }
      );

      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
    });

    it('複数フラグが全て満たされる場合、表示されること', () => {
      render(
        <FlagGuard flags={['beta', 'experimental']}>
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper(['beta', 'experimental', 'alpha']) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });
  });

  describe('空のフラグ配列', () => {
    it('空のフラグ配列の場合、子要素が表示されること', () => {
      render(
        <FlagGuard flags={[]}>
          <div data-testid="protected-content">Protected Content</div>
        </FlagGuard>,
        { wrapper: createWrapper([]) }
      );

      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });
  });
});

describe('useHasFlag', () => {
  it('フラグがある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasFlag('beta'), {
      wrapper: createWrapper(['beta', 'alpha']),
    });

    expect(result.current).toBe(true);
  });

  it('フラグがない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasFlag('experimental'), {
      wrapper: createWrapper(['beta', 'alpha']),
    });

    expect(result.current).toBe(false);
  });
});

describe('useHasAllFlags', () => {
  it('全てのフラグがある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAllFlags(['beta', 'alpha']), {
      wrapper: createWrapper(['beta', 'alpha', 'experimental']),
    });

    expect(result.current).toBe(true);
  });

  it('一部のフラグがない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAllFlags(['beta', 'experimental']), {
      wrapper: createWrapper(['beta', 'alpha']),
    });

    expect(result.current).toBe(false);
  });

  it('空の配列の場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAllFlags([]), {
      wrapper: createWrapper(['beta']),
    });

    expect(result.current).toBe(true);
  });
});

describe('useHasAnyFlag', () => {
  it('いずれかのフラグがある場合 true を返すこと', () => {
    const { result } = renderHook(() => useHasAnyFlag(['experimental', 'beta']), {
      wrapper: createWrapper(['beta']),
    });

    expect(result.current).toBe(true);
  });

  it('全てのフラグがない場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAnyFlag(['experimental', 'gamma']), {
      wrapper: createWrapper(['beta', 'alpha']),
    });

    expect(result.current).toBe(false);
  });

  it('空の配列の場合 false を返すこと', () => {
    const { result } = renderHook(() => useHasAnyFlag([]), {
      wrapper: createWrapper(['beta']),
    });

    expect(result.current).toBe(false);
  });
});
