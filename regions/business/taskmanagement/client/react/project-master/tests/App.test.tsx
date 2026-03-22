import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { QueryClientProvider } from '@tanstack/react-query';
import { QueryClient } from '@tanstack/react-query';
import { server } from './testutil/msw-setup';
import { ProjectTypeList } from '../src/features/project-types/ProjectTypeList';

// MSWサーバーのライフサイクル管理
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// テスト用のQueryClientラッパー: 各テストで独立したキャッシュを使用
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
}

// テスト用ラッパーコンポーネント
function TestWrapper({ children }: { children: React.ReactNode }) {
  const queryClient = createTestQueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe('ProjectTypeList', () => {
  // プロジェクトタイプ一覧が正常にレンダリングされることを確認
  it('プロジェクトタイプ一覧を表示する', async () => {
    render(
      <TestWrapper>
        <ProjectTypeList />
      </TestWrapper>
    );

    // ローディング表示の確認
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();

    // データ取得後にプロジェクトタイプが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('ソフトウェア開発')).toBeInTheDocument();
      expect(screen.getByText('インフラ構築')).toBeInTheDocument();
    });
  });

  // ページタイトルが表示されることを確認
  it('ページタイトルを表示する', async () => {
    render(
      <TestWrapper>
        <ProjectTypeList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('プロジェクトタイプ一覧')).toBeInTheDocument();
    });
  });

  // 新規作成ボタンが存在することを確認
  it('新規作成ボタンを表示する', async () => {
    render(
      <TestWrapper>
        <ProjectTypeList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('新規作成')).toBeInTheDocument();
    });
  });
});
