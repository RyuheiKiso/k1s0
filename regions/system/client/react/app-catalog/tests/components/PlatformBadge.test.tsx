import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { PlatformBadge } from '../../src/components/PlatformBadge';

describe('PlatformBadge', () => {
  it('windows プラットフォームのラベルを表示する', () => {
    render(<PlatformBadge platform="windows" />);
    expect(screen.getByText('Windows')).toBeInTheDocument();
  });

  it('linux プラットフォームのラベルを表示する', () => {
    render(<PlatformBadge platform="linux" />);
    expect(screen.getByText('Linux')).toBeInTheDocument();
  });

  it('macos プラットフォームのラベルを表示する', () => {
    render(<PlatformBadge platform="macos" />);
    expect(screen.getByText('macOS')).toBeInTheDocument();
  });

  it('data-platform 属性が正しく設定される', () => {
    render(<PlatformBadge platform="windows" />);
    const badge = document.querySelector('[data-platform="windows"]');
    expect(badge).toBeInTheDocument();
  });

  it('platform-badge クラスが付与される', () => {
    render(<PlatformBadge platform="linux" />);
    const badge = document.querySelector('.platform-badge');
    expect(badge).toBeInTheDocument();
  });

  it('アイコン要素が含まれる', () => {
    render(<PlatformBadge platform="macos" />);
    const icon = document.querySelector('.platform-badge__icon');
    expect(icon).toBeInTheDocument();
  });
});
