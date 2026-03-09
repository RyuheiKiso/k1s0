import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { AuthContext, type AuthContextValue } from './AuthContext';
import { useAuth } from './useAuth';

function createMockAuthValue(overrides: Partial<AuthContextValue> = {}): AuthContextValue {
  return {
    user: null,
    isAuthenticated: false,
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

  it('login を呼び出せる', async () => {
    const loginMock = vi.fn();
    const mockValue = createMockAuthValue({ login: loginMock });
    const { result } = renderHook(() => useAuth(), { wrapper: createWrapper(mockValue) });

    await act(async () => {
      await result.current.login({ username: 'user@example.com', password: 'password' });
    });

    expect(loginMock).toHaveBeenCalledWith({ username: 'user@example.com', password: 'password' });
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
    expect(() => {
      renderHook(() => useAuth());
    }).toThrow('useAuth は AuthProvider の内部で使用する必要があります');
  });
});
