import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import { AuthProvider } from '../auth/AuthProvider';
import { ProtectedRoute } from './ProtectedRoute';

describe('ProtectedRoute', () => {
  it('未認証時は fallback を表示する', () => {
    render(
      <AuthProvider>
        <ProtectedRoute fallback={<div>ログインページ</div>}>
          <div>保護されたコンテンツ</div>
        </ProtectedRoute>
      </AuthProvider>
    );

    expect(screen.getByText('ログインページ')).toBeInTheDocument();
    expect(screen.queryByText('保護されたコンテンツ')).not.toBeInTheDocument();
  });
});
