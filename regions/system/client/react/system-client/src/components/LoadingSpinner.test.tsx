import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LoadingSpinner } from './LoadingSpinner';

describe('LoadingSpinner', () => {
  it('role="status" が設定される', () => {
    render(<LoadingSpinner />);
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('メッセージ未指定時は aria-label が "Loading" になる', () => {
    render(<LoadingSpinner />);
    expect(screen.getByRole('status')).toHaveAttribute('aria-label', 'Loading');
  });

  it('メッセージ指定時は aria-label にメッセージが設定される', () => {
    render(<LoadingSpinner message="読み込み中..." />);
    expect(screen.getByRole('status')).toHaveAttribute('aria-label', '読み込み中...');
  });

  it('メッセージテキストが表示される', () => {
    render(<LoadingSpinner message="データを取得中" />);
    expect(screen.getByText('データを取得中')).toBeInTheDocument();
  });

  it('メッセージ未指定時はテキストが表示されない', () => {
    render(<LoadingSpinner />);
    // p要素が存在しないことを確認
    expect(screen.getByRole('status').querySelector('p')).toBeNull();
  });
});
