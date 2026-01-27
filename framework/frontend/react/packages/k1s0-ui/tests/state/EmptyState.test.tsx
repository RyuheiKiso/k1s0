/**
 * EmptyState コンポーネントのテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import {
  EmptyState,
  NoData,
  NoSearchResults,
  ErrorState,
} from '../../src/state/EmptyState';

describe('EmptyState', () => {
  it('タイトルを表示すること', () => {
    render(<EmptyState title="No items found" />);

    expect(screen.getByText('No items found')).toBeInTheDocument();
  });

  it('説明を表示すること', () => {
    render(
      <EmptyState
        title="No items found"
        description="Try adding some items to get started."
      />
    );

    expect(screen.getByText('Try adding some items to get started.')).toBeInTheDocument();
  });

  it('アイコンを表示すること', () => {
    render(
      <EmptyState
        title="No items"
        icon={<span data-testid="custom-icon">Icon</span>}
      />
    );

    expect(screen.getByTestId('custom-icon')).toBeInTheDocument();
  });

  it('アクションボタンを表示すること', () => {
    const handleAction = vi.fn();

    render(
      <EmptyState
        title="No items"
        actionLabel="Add Item"
        onAction={handleAction}
      />
    );

    const button = screen.getByRole('button', { name: 'Add Item' });
    expect(button).toBeInTheDocument();
  });

  it('アクションボタンをクリックするとコールバックが呼ばれること', () => {
    const handleAction = vi.fn();

    render(
      <EmptyState
        title="No items"
        actionLabel="Add Item"
        onAction={handleAction}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: 'Add Item' }));
    expect(handleAction).toHaveBeenCalledTimes(1);
  });

  it('actionLabel がない場合はボタンを表示しないこと', () => {
    const handleAction = vi.fn();

    render(
      <EmptyState
        title="No items"
        onAction={handleAction}
      />
    );

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('onAction がない場合はボタンを表示しないこと', () => {
    render(
      <EmptyState
        title="No items"
        actionLabel="Add Item"
      />
    );

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('children を表示すること', () => {
    render(
      <EmptyState title="No items">
        <div data-testid="custom-child">Custom Content</div>
      </EmptyState>
    );

    expect(screen.getByTestId('custom-child')).toBeInTheDocument();
  });

  it('カスタム sx スタイルを適用できること', () => {
    const { container } = render(
      <EmptyState
        title="No items"
        sx={{ backgroundColor: 'red' }}
      />
    );

    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveStyle({ backgroundColor: 'red' });
  });
});

describe('NoData', () => {
  it('デフォルトメッセージを表示すること', () => {
    render(<NoData />);

    expect(screen.getByText('データがありません')).toBeInTheDocument();
  });

  it('カスタムメッセージを表示できること', () => {
    render(<NoData message="No items available" />);

    expect(screen.getByText('No items available')).toBeInTheDocument();
  });
});

describe('NoSearchResults', () => {
  it('基本的なメッセージを表示すること', () => {
    render(<NoSearchResults />);

    expect(screen.getByText('検索結果がありません')).toBeInTheDocument();
  });

  it('検索クエリを含むメッセージを表示すること', () => {
    render(<NoSearchResults query="test query" />);

    expect(screen.getByText(/「test query」に一致する結果が見つかりませんでした/)).toBeInTheDocument();
  });

  it('リセットボタンを表示すること', () => {
    const handleReset = vi.fn();

    render(<NoSearchResults onReset={handleReset} />);

    expect(screen.getByRole('button', { name: '検索条件をリセット' })).toBeInTheDocument();
  });

  it('リセットボタンをクリックするとコールバックが呼ばれること', () => {
    const handleReset = vi.fn();

    render(<NoSearchResults onReset={handleReset} />);

    fireEvent.click(screen.getByRole('button', { name: '検索条件をリセット' }));
    expect(handleReset).toHaveBeenCalledTimes(1);
  });

  it('onReset がない場合はリセットボタンを表示しないこと', () => {
    render(<NoSearchResults query="test" />);

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

describe('ErrorState', () => {
  it('デフォルトのエラーメッセージを表示すること', () => {
    render(<ErrorState />);

    expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
    expect(screen.getByText('エラーが発生しました', { selector: 'p' })).toBeInTheDocument();
  });

  it('カスタムエラーメッセージを表示すること', () => {
    render(<ErrorState message="Something went wrong" />);

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('再試行ボタンを表示すること', () => {
    const handleRetry = vi.fn();

    render(<ErrorState onRetry={handleRetry} />);

    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument();
  });

  it('再試行ボタンをクリックするとコールバックが呼ばれること', () => {
    const handleRetry = vi.fn();

    render(<ErrorState onRetry={handleRetry} />);

    fireEvent.click(screen.getByRole('button', { name: '再試行' }));
    expect(handleRetry).toHaveBeenCalledTimes(1);
  });

  it('onRetry がない場合は再試行ボタンを表示しないこと', () => {
    render(<ErrorState message="Error" />);

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
