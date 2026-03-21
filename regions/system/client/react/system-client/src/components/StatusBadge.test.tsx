import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StatusBadge } from './StatusBadge';

describe('StatusBadge', () => {
  it('ラベルテキストを表示する', () => {
    render(<StatusBadge label="有効" variant="success" />);
    expect(screen.getByText('有効')).toBeInTheDocument();
  });

  it('role="status" が設定される', () => {
    render(<StatusBadge label="警告" variant="warning" />);
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('aria-label にラベルが設定される', () => {
    render(<StatusBadge label="エラー" variant="danger" />);
    expect(screen.getByRole('status')).toHaveAttribute('aria-label', 'エラー');
  });

  it('success バリアントは緑色系のスタイルが適用される', () => {
    render(<StatusBadge label="成功" variant="success" />);
    const badge = screen.getByRole('status');
    expect(badge).toHaveStyle({ backgroundColor: '#d4edda', color: '#155724' });
  });

  it('warning バリアントは黄色系のスタイルが適用される', () => {
    render(<StatusBadge label="警告" variant="warning" />);
    const badge = screen.getByRole('status');
    expect(badge).toHaveStyle({ backgroundColor: '#fff3cd', color: '#856404' });
  });

  it('danger バリアントは赤色系のスタイルが適用される', () => {
    render(<StatusBadge label="危険" variant="danger" />);
    const badge = screen.getByRole('status');
    expect(badge).toHaveStyle({ backgroundColor: '#f8d7da', color: '#721c24' });
  });

  it('info バリアントは青色系のスタイルが適用される', () => {
    render(<StatusBadge label="情報" variant="info" />);
    const badge = screen.getByRole('status');
    expect(badge).toHaveStyle({ backgroundColor: '#cce5ff', color: '#004085' });
  });

  it('neutral バリアントはグレー系のスタイルが適用される', () => {
    render(<StatusBadge label="無効" variant="neutral" />);
    const badge = screen.getByRole('status');
    expect(badge).toHaveStyle({ backgroundColor: '#e2e3e5', color: '#383d41' });
  });
});
