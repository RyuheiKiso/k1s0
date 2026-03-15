import { describe, it, expect, afterEach, afterAll, beforeAll, vi } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import React from 'react';
import { AuthProvider } from './AuthProvider';
import { useAuth } from './useAuth';

const API_BASE = 'http://localhost:3000/bff';

const server = setupServer(
  // デフォルト: セッションなし（401 を返す）
  http.get(`${API_BASE}/auth/session`, () => {
    return new HttpResponse(null, { status: 401 });
  }),
  http.post(`${API_BASE}/auth/logout`, () => {
    return new HttpResponse(null, { status: 204 });
  }),
);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <AuthProvider apiBaseURL={API_BASE}>{children}</AuthProvider>
);

describe('AuthProvider（BFF セッション統合）', () => {
  it('初期化時にセッション確認を行い、未認証状態になる', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
    });
  });

  it('既存セッションがある場合は認証済みになる', async () => {
    server.use(
      http.get(`${API_BASE}/auth/session`, () => {
        return HttpResponse.json({
          id: 'user-sub-001',
          authenticated: true,
          csrf_token: 'csrf-token-abc',
        });
      }),
    );

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.user?.id).toBe('user-sub-001');
    });
  });

  it('login は BFF の /auth/login へリダイレクトする', async () => {
    // window.location.href の設定を検証するため、モックを用意
    const originalLocation = window.location;
    const locationMock = { ...originalLocation, href: '' };
    Object.defineProperty(window, 'location', {
      value: locationMock,
      writable: true,
    });

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(false);
    });

    act(() => {
      result.current.login();
    });

    expect(window.location.href).toBe(`${API_BASE}/auth/login`);

    // window.location を復元する
    Object.defineProperty(window, 'location', {
      value: originalLocation,
      writable: true,
    });
  });

  it('logout で API を呼び出してユーザー情報をクリアする', async () => {
    // まず認証済み状態にする
    server.use(
      http.get(`${API_BASE}/auth/session`, () => {
        return HttpResponse.json({
          id: 'user-sub-001',
          authenticated: true,
          csrf_token: 'csrf-token-abc',
        });
      }),
    );

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(true);
    });

    await act(async () => {
      await result.current.logout();
    });

    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBeNull();
  });
});
