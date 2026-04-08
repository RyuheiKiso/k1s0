import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AuthContext, type AuthContextValue } from '../auth/AuthContext';
import { ProtectedRoute } from './ProtectedRoute';
// L-004 監査対応: RolesCondition 型のインポート（型チェック用）
import type { RolesCondition } from './ProtectedRoute';

function createMockAuthValue(overrides: Partial<AuthContextValue> = {}): AuthContextValue {
  return {
    user: null,
    isAuthenticated: false,
    // CRIT-003 監査対応: AuthContextValue の loading フィールドは必須のため false を設定する
    loading: false,
    login: vi.fn(),
    logout: vi.fn(),
    ...overrides,
  };
}

describe('ProtectedRoute', () => {
  it('未認証時は fallback を表示する', () => {
    const mockValue = createMockAuthValue({ isAuthenticated: false });
    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>ログインページ</div>}>
          <div>保護されたコンテンツ</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('ログインページ')).toBeInTheDocument();
    expect(screen.queryByText('保護されたコンテンツ')).not.toBeInTheDocument();
  });

  it('認証済みの場合は children を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['admin'] },
    });
    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>ログインページ</div>}>
          <div>保護されたコンテンツ</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('保護されたコンテンツ')).toBeInTheDocument();
    expect(screen.queryByText('ログインページ')).not.toBeInTheDocument();
  });

  it('必要ロールが不足している場合は fallback を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['user'] },
    });

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} requiredRoles={['admin']}>
          <div>管理画面</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('権限がありません')).toBeInTheDocument();
    expect(screen.queryByText('管理画面')).not.toBeInTheDocument();
  });

  // L-004 監査対応: roles プロパティの AND/OR 条件テスト
  it('roles.required（AND条件）を全て保持している場合は children を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['admin', 'editor'] },
    });
    const condition: RolesCondition = { required: ['admin', 'editor'] };

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} roles={condition}>
          <div>管理画面</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('管理画面')).toBeInTheDocument();
    expect(screen.queryByText('権限がありません')).not.toBeInTheDocument();
  });

  it('roles.required（AND条件）の一部しか保持していない場合は fallback を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['admin'] },
    });
    const condition: RolesCondition = { required: ['admin', 'editor'] };

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} roles={condition}>
          <div>管理画面</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('権限がありません')).toBeInTheDocument();
    expect(screen.queryByText('管理画面')).not.toBeInTheDocument();
  });

  it('roles.any（OR条件）のいずれかを保持している場合は children を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['editor'] },
    });
    const condition: RolesCondition = { any: ['admin', 'editor'] };

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} roles={condition}>
          <div>コンテンツ</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('コンテンツ')).toBeInTheDocument();
    expect(screen.queryByText('権限がありません')).not.toBeInTheDocument();
  });

  it('roles.required と roles.any の両方を満たす場合に children を表示する（AND/OR 複合条件）', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['admin', 'editor'] },
    });
    // required: admin を必ず持つ、かつ any: editor または viewer のいずれかを持つ
    const condition: RolesCondition = { required: ['admin'], any: ['editor', 'viewer'] };

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} roles={condition}>
          <div>複合条件コンテンツ</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('複合条件コンテンツ')).toBeInTheDocument();
  });

  it('roles.required を満たすが roles.any を満たさない場合は fallback を表示する', () => {
    const mockValue = createMockAuthValue({
      isAuthenticated: true,
      user: { id: 'user-1', username: 'test@example.com', roles: ['admin'] },
    });
    const condition: RolesCondition = { required: ['admin'], any: ['editor', 'viewer'] };

    render(
      <AuthContext.Provider value={mockValue}>
        <ProtectedRoute fallback={<div>権限がありません</div>} roles={condition}>
          <div>複合条件コンテンツ</div>
        </ProtectedRoute>
      </AuthContext.Provider>
    );

    expect(screen.getByText('権限がありません')).toBeInTheDocument();
    expect(screen.queryByText('複合条件コンテンツ')).not.toBeInTheDocument();
  });
});
