import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { AppButton } from './AppButton';

describe('AppButton', () => {
  it('ラベルを表示する', () => {
    render(<AppButton label="クリック" onClick={() => {}} />);
    expect(screen.getByText('クリック')).toBeInTheDocument();
  });

  it('クリックイベントが発火する', () => {
    const onClick = vi.fn();
    render(<AppButton label="クリック" onClick={onClick} />);
    fireEvent.click(screen.getByText('クリック'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('isLoading が true の場合はボタンが無効になる', () => {
    render(<AppButton label="クリック" onClick={() => {}} isLoading />);
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
