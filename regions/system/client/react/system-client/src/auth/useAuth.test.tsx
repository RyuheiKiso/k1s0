import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { type ReactNode } from 'react';
import { AuthContext, type AuthContextValue } from './AuthContext';
import { useAuth } from './useAuth';

function createMockAuthValue(overrides: Partial<AuthContextValue> = {}): AuthContextValue {
  return {
    user: null,
    isAuthenticated: false,
    // CRIT-003 監査対応: AuthContextValue の loading フィールドを追加（型整合性確保）
    loading: false,
    login: vi.fn(),
    logout: vi.fn(),
    ...overrides,
  };
}

function createWrapper(value: AuthContextValue) {
  return ({ children }: { children: ReactNode }) => (
    <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
  );
}

describe('useAuth', () => {
  it('初期状態は unauthenticated', () => {
    const mockValue = createMockAuthValue();
    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper(mockValue) });
    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBeNull();
  });

  it('login を呼び出せる（引数なしのリダイレクト型）', () => {
    const loginMock = vi.fn();
    const mockValue = createMockAuthValue({ login: loginMock });
    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper(mockValue) });

    act(() => {
      result.current.login();
    });

    expect(loginMock).toHaveBeenCalled();
  });

  it('logout を呼び出せる', async () => {
    const logoutMock = vi.fn();
    const mockValue = createMockAuthValue({
      user: { id: 'user-1', username: 'user@example.com' },
      isAuthenticated: true,
      logout: logoutMock,
    });
    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper(mockValue) });

    await act(async () => {
      await result.current.logout();
    });

    expect(logoutMock).toHaveBeenCalled();
  });

  it('AuthProvider の外で使用するとエラーになる', () => {
    // HIGH-012 監査対応: DOCS-MED-005 でデフォルトエラーメッセージが英語に統一されたためテストを更新
    expect(() => {
      renderHook(() => useAuth());
    }).toThrow('useAuth must be used within an AuthProvider');
  });
});
