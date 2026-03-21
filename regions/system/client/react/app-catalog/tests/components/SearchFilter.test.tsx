import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SearchFilter } from '../../src/components/SearchFilter';

// デフォルトのプロップスを生成するファクトリ関数
function buildProps(overrides = {}) {
  return {
    query: '',
    onSearch: vi.fn(),
    onCategoryChange: vi.fn(),
    onPlatformChange: vi.fn(),
    categories: ['ツール', '開発', 'セキュリティ'],
    selectedCategory: '',
    selectedPlatform: '' as const,
    ...overrides,
  };
}

describe('SearchFilter', () => {
  it('検索入力欄がレンダリングされる', () => {
    render(<SearchFilter {...buildProps()} />);
    expect(screen.getByLabelText('アプリを検索')).toBeInTheDocument();
  });

  it('初期クエリ値が入力欄に反映される', () => {
    render(<SearchFilter {...buildProps({ query: 'test' })} />);
    expect(screen.getByLabelText('アプリを検索')).toHaveValue('test');
  });

  it('入力変更時に onSearch が呼ばれる', () => {
    const onSearch = vi.fn();
    render(<SearchFilter {...buildProps({ onSearch })} />);
    fireEvent.change(screen.getByLabelText('アプリを検索'), {
      target: { value: 'new query' },
    });
    expect(onSearch).toHaveBeenCalledWith('new query');
  });

  it('カテゴリセレクトがレンダリングされる', () => {
    render(<SearchFilter {...buildProps()} />);
    expect(screen.getByLabelText('カテゴリで絞り込み')).toBeInTheDocument();
  });

  it('カテゴリ一覧がオプションとして表示される', () => {
    render(<SearchFilter {...buildProps()} />);
    expect(screen.getByRole('option', { name: 'ツール' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: '開発' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'セキュリティ' })).toBeInTheDocument();
  });

  it('カテゴリ変更時に onCategoryChange が呼ばれる', () => {
    const onCategoryChange = vi.fn();
    render(<SearchFilter {...buildProps({ onCategoryChange })} />);
    fireEvent.change(screen.getByLabelText('カテゴリで絞り込み'), {
      target: { value: 'ツール' },
    });
    expect(onCategoryChange).toHaveBeenCalledWith('ツール');
  });

  it('OS セレクトがレンダリングされる', () => {
    render(<SearchFilter {...buildProps()} />);
    expect(screen.getByLabelText('OSで絞り込み')).toBeInTheDocument();
  });

  it('OS セレクトに Windows / macOS / Linux の選択肢がある', () => {
    render(<SearchFilter {...buildProps()} />);
    expect(screen.getByRole('option', { name: 'Windows' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'macOS' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Linux' })).toBeInTheDocument();
  });

  it('OS 変更時に onPlatformChange が呼ばれる', () => {
    const onPlatformChange = vi.fn();
    render(<SearchFilter {...buildProps({ onPlatformChange })} />);
    fireEvent.change(screen.getByLabelText('OSで絞り込み'), {
      target: { value: 'linux' },
    });
    expect(onPlatformChange).toHaveBeenCalledWith('linux');
  });

  it('選択中のカテゴリが反映される', () => {
    render(<SearchFilter {...buildProps({ selectedCategory: 'ツール' })} />);
    expect(screen.getByLabelText('カテゴリで絞り込み')).toHaveValue('ツール');
  });

  it('選択中の OS が反映される', () => {
    render(<SearchFilter {...buildProps({ selectedPlatform: 'windows' as const })} />);
    expect(screen.getByLabelText('OSで絞り込み')).toHaveValue('windows');
  });
});
