import { describe, it, expect, afterEach, afterAll, beforeAll } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import React from 'react';
import { AuthProvider } from './AuthProvider';
import { useAuth } from './useAuth';

const API_BASE = 'http://localhost:3000/bff';

const server = setupServer(
  // デフォルト: セッションなし
  http.get(`${API_BASE}/auth/me`, () => {
    return new HttpResponse(null, { status: 401 });
  }),
  http.post(`${API_BASE}/auth/login`, async ({ request }) => {
    const body = (await request.json()) as { username: string; password: string };
    return HttpResponse.json({
      id: 'user-1',
      username: body.username,
    });
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

describe('AuthProvider（API 統合）', () => {
  it('初期化時にセッション確認を行い、未認証状態になる', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
    });
  });

  it('既存セッションがある場合は認証済みになる', async () => {
    server.use(
      http.get(`${API_BASE}/auth/me`, () => {
        return HttpResponse.json({ id: 'user-existing', username: 'existing@example.com' });
      }),
    );

    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.user?.username).toBe('existing@example.com');
    });
  });

  it('login で API を呼び出してユーザー情報を設定する', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    // 初期化完了を待つ
    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(false);
    });

    await act(async () => {
      await result.current.login({
        username: 'user@example.com',
        password: 'password123',
      });
    });

    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.user?.id).toBe('user-1');
    expect(result.current.user?.username).toBe('user@example.com');
  });

  it('logout で API を呼び出してユーザー情報をクリアする', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });

    await waitFor(() => {
      expect(result.current.isAuthenticated).toBe(false);
    });

    await act(async () => {
      await result.current.login({
        username: 'user@example.com',
        password: 'password123',
      });
    });

    expect(result.current.isAuthenticated).toBe(true);

    await act(async () => {
      await result.current.logout();
    });

    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.user).toBeNull();
  });
});
