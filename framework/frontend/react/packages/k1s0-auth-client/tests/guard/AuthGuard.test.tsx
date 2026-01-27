/**
 * AuthGuard コンポーネントのテスト
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import React, { type ReactNode } from 'react';
import { AuthGuard, RequireAuth, RequireRole, RequirePermission } from '../../src/guard/AuthGuard';
import { AuthProvider, useAuth } from '../../src/provider/AuthContext';
import type { AuthState, AuthClientConfig, TokenPair } from '../../src/types';

// AuthProvider のモック版コンテキスト
const mockAuthContext = {
  state: {
    status: 'authenticated',
    isAuthenticated: true,
    isLoading: false,
    user: {
      id: 'user-123',
      roles: ['admin', 'user'],
      permissions: ['read', 'write'],
      tenantId: 'tenant-1',
      claims: {} as any,
    },
  } as AuthState,
  hasAnyRole: vi.fn((roles: string[]) => roles.some((r) => ['admin', 'user'].includes(r))),
  hasAllPermissions: vi.fn((permissions: string[]) =>
    permissions.every((p) => ['read', 'write'].includes(p))
  ),
};

// useAuth のモック
vi.mock('../../src/provider/AuthContext', async () => {
  const actual = await vi.importActual('../../src/provider/AuthContext');
  return {
    ...actual,
    useAuth: vi.fn(),
  };
});

describe('AuthGuard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);
  });

  describe('認証状態', () => {
    it('認証済みの場合、子要素を表示すること', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        state: { ...mockAuthContext.state, status: 'authenticated', isAuthenticated: true },
      });

      render(
        <AuthGuard>
          <div data-testid="protected-content">Protected Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('未認証の場合、子要素を表示しないこと', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        state: { status: 'unauthenticated', isAuthenticated: false, isLoading: false },
      });

      render(
        <AuthGuard>
          <div data-testid="protected-content">Protected Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      });
    });

    it('ローディング中はローディングコンポーネントを表示すること', () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        state: { status: 'loading', isAuthenticated: false, isLoading: true },
      });

      render(
        <AuthGuard loadingComponent={<div data-testid="loading">Loading...</div>}>
          <div data-testid="protected-content">Protected Content</div>
        </AuthGuard>
      );

      expect(screen.getByTestId('loading')).toBeInTheDocument();
      expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
    });

    it('未認証時に unauthenticatedComponent を表示すること', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        state: { status: 'unauthenticated', isAuthenticated: false, isLoading: false },
      });

      render(
        <AuthGuard
          unauthenticatedComponent={<div data-testid="unauthenticated">Please login</div>}
        >
          <div data-testid="protected-content">Protected Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('unauthenticated')).toBeInTheDocument();
      });
    });
  });

  describe('ロールベースの認可', () => {
    it('必要なロールがある場合、子要素を表示すること', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAnyRole: vi.fn(() => true),
      });

      render(
        <AuthGuard roles={['admin']}>
          <div data-testid="protected-content">Admin Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('必要なロールがない場合、子要素を表示しないこと', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAnyRole: vi.fn(() => false),
      });

      render(
        <AuthGuard roles={['superadmin']}>
          <div data-testid="protected-content">Admin Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      });
    });

    it('権限不足時に forbiddenComponent を表示すること', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAnyRole: vi.fn(() => false),
      });

      render(
        <AuthGuard
          roles={['superadmin']}
          forbiddenComponent={<div data-testid="forbidden">Access Denied</div>}
        >
          <div data-testid="protected-content">Admin Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('forbidden')).toBeInTheDocument();
      });
    });
  });

  describe('パーミッションベースの認可', () => {
    it('必要なパーミッションがある場合、子要素を表示すること', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAllPermissions: vi.fn(() => true),
      });

      render(
        <AuthGuard permissions={['read', 'write']}>
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('必要なパーミッションがない場合、子要素を表示しないこと', async () => {
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAllPermissions: vi.fn(() => false),
      });

      render(
        <AuthGuard permissions={['delete']}>
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      });
    });
  });

  describe('カスタム認可関数', () => {
    it('カスタム認可関数が true を返す場合、子要素を表示すること', async () => {
      const authorize = vi.fn().mockResolvedValue(true);
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);

      render(
        <AuthGuard authorize={authorize}>
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.getByTestId('protected-content')).toBeInTheDocument();
      });
    });

    it('カスタム認可関数が false を返す場合、子要素を表示しないこと', async () => {
      const authorize = vi.fn().mockResolvedValue(false);
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);

      render(
        <AuthGuard authorize={authorize}>
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(screen.queryByTestId('protected-content')).not.toBeInTheDocument();
      });
    });
  });

  describe('リダイレクト', () => {
    it('未認証時に redirectTo へ遷移すること', async () => {
      const navigate = vi.fn();
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        state: { status: 'unauthenticated', isAuthenticated: false, isLoading: false },
      });

      render(
        <AuthGuard redirectTo="/login" navigate={navigate}>
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(navigate).toHaveBeenCalledWith('/login');
      });
    });

    it('権限不足時に forbiddenRedirectTo へ遷移すること', async () => {
      const navigate = vi.fn();
      (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
        ...mockAuthContext,
        hasAnyRole: vi.fn(() => false),
      });

      render(
        <AuthGuard
          roles={['superadmin']}
          forbiddenRedirectTo="/forbidden"
          navigate={navigate}
        >
          <div data-testid="protected-content">Content</div>
        </AuthGuard>
      );

      await waitFor(() => {
        expect(navigate).toHaveBeenCalledWith('/forbidden');
      });
    });
  });
});

describe('RequireAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);
  });

  it('認証済みの場合、子要素を表示すること', async () => {
    render(
      <RequireAuth>
        <div data-testid="protected-content">Content</div>
      </RequireAuth>
    );

    await waitFor(() => {
      expect(screen.getByTestId('protected-content')).toBeInTheDocument();
    });
  });

  it('デフォルトで /login にリダイレクトすること', async () => {
    const navigate = vi.fn();
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
      ...mockAuthContext,
      state: { status: 'unauthenticated', isAuthenticated: false, isLoading: false },
    });

    render(
      <RequireAuth navigate={navigate}>
        <div>Content</div>
      </RequireAuth>
    );

    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith('/login');
    });
  });
});

describe('RequireRole', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);
  });

  it('単一ロールを指定できること', async () => {
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
      ...mockAuthContext,
      hasAnyRole: vi.fn((roles) => roles.includes('admin')),
    });

    render(
      <RequireRole role="admin">
        <div data-testid="content">Admin Content</div>
      </RequireRole>
    );

    await waitFor(() => {
      expect(screen.getByTestId('content')).toBeInTheDocument();
    });
  });
});

describe('RequirePermission', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue(mockAuthContext);
  });

  it('単一パーミッションを指定できること', async () => {
    (useAuth as ReturnType<typeof vi.fn>).mockReturnValue({
      ...mockAuthContext,
      hasAllPermissions: vi.fn((perms) => perms.every((p: string) => ['read'].includes(p))),
    });

    render(
      <RequirePermission permission="read">
        <div data-testid="content">Content</div>
      </RequirePermission>
    );

    await waitFor(() => {
      expect(screen.getByTestId('content')).toBeInTheDocument();
    });
  });
});
