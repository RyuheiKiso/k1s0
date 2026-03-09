import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { AuthContext, type AuthContextValue } from '../auth/AuthContext';
import { ProtectedRoute } from './ProtectedRoute';

function createMockAuthValue(overrides: Partial<AuthContextValue> = {}): AuthContextValue {
  return {
    user: null,
    isAuthenticated: false,
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
      user: { id: 'user-1', username: 'test@example.com' },
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
});
