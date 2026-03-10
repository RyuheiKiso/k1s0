import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { AppCard } from '../../src/components/AppCard';
import type { App } from '../../src/api/types';

const mockApp: App = {
  id: 'app-1',
  name: 'テストアプリ',
  description: 'テスト用のアプリケーション',
  category: 'ツール',
  icon_url: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

function renderWithRouter(ui: React.ReactElement) {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
}

describe('AppCard', () => {
  it('アプリ名を表示する', () => {
    renderWithRouter(<AppCard app={mockApp} />);
    expect(screen.getByText('テストアプリ')).toBeInTheDocument();
  });

  it('説明を表示する', () => {
    renderWithRouter(<AppCard app={mockApp} />);
    expect(screen.getByText('テスト用のアプリケーション')).toBeInTheDocument();
  });

  it('カテゴリを表示する', () => {
    renderWithRouter(<AppCard app={mockApp} />);
    expect(screen.getByText('ツール')).toBeInTheDocument();
  });

  it('最新バージョンを表示する', () => {
    renderWithRouter(<AppCard app={mockApp} latestVersion="1.2.3" />);
    expect(screen.getByText('v1.2.3')).toBeInTheDocument();
  });

  it('アイコンがない場合はプレースホルダーを表示する', () => {
    renderWithRouter(<AppCard app={mockApp} />);
    expect(screen.getByText('テ')).toBeInTheDocument();
  });

  it('詳細ページへのリンクを持つ', () => {
    renderWithRouter(<AppCard app={mockApp} />);
    const link = screen.getByTestId('app-card');
    expect(link).toHaveAttribute('href', '/apps/app-1');
  });
});
