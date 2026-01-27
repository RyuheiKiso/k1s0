/**
 * Loading コンポーネントのテスト
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import React from 'react';
import {
  LoadingSpinner,
  LoadingBar,
  SkeletonLoader,
  PageLoading,
} from '../../src/state/Loading';

describe('LoadingSpinner', () => {
  it('基本的なスピナーをレンダリングすること', () => {
    render(<LoadingSpinner />);

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('メッセージを表示できること', () => {
    render(<LoadingSpinner message="Loading..." />);

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('centered プロパティでセンタリングされること', () => {
    const { container } = render(<LoadingSpinner centered />);

    // センタリング用のスタイルが適用されていることを確認
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveStyle({ display: 'flex' });
  });

  it('overlay プロパティでオーバーレイとして表示されること', () => {
    const { container } = render(<LoadingSpinner overlay />);

    // オーバーレイ用のスタイルが適用されていることを確認
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveStyle({ position: 'absolute' });
  });

  it('size プロパティでサイズを変更できること', () => {
    render(<LoadingSpinner size={60} />);

    const spinner = screen.getByRole('progressbar');
    expect(spinner).toHaveStyle({ width: '60px', height: '60px' });
  });
});

describe('LoadingBar', () => {
  it('loading が true の場合に表示されること', () => {
    render(<LoadingBar loading />);

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('loading が false の場合に表示されないこと', () => {
    render(<LoadingBar loading={false} />);

    expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
  });

  it('デフォルトで loading が true であること', () => {
    render(<LoadingBar />);

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });
});

describe('SkeletonLoader', () => {
  describe('lines モード', () => {
    it('指定した行数のスケルトンを表示すること', () => {
      const { container } = render(<SkeletonLoader lines={3} />);

      const skeletons = container.querySelectorAll('.MuiSkeleton-root');
      expect(skeletons.length).toBe(3);
    });

    it('デフォルトで3行表示すること', () => {
      const { container } = render(<SkeletonLoader />);

      const skeletons = container.querySelectorAll('.MuiSkeleton-root');
      expect(skeletons.length).toBe(3);
    });
  });

  describe('avatar モード', () => {
    it('アバター付きのスケルトンを表示すること', () => {
      const { container } = render(<SkeletonLoader avatar lines={2} />);

      // アバター用の円形スケルトン + テキスト行
      const skeletons = container.querySelectorAll('.MuiSkeleton-root');
      expect(skeletons.length).toBe(3); // 1 avatar + 2 lines
    });
  });

  describe('card モード', () => {
    it('カード形式のスケルトンを表示すること', () => {
      const { container } = render(<SkeletonLoader card />);

      // 画像部分 + テキスト行2つ
      const skeletons = container.querySelectorAll('.MuiSkeleton-root');
      expect(skeletons.length).toBe(3);
    });
  });
});

describe('PageLoading', () => {
  it('デフォルトメッセージを表示すること', () => {
    render(<PageLoading />);

    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('カスタムメッセージを表示できること', () => {
    render(<PageLoading message="データを読み込んでいます..." />);

    expect(screen.getByText('データを読み込んでいます...')).toBeInTheDocument();
  });
});
