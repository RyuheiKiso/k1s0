import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { QueryClientProvider } from '@tanstack/react-query';
import { QueryClient } from '@tanstack/react-query';
import { server } from './testutil/msw-setup';
import { OrderList } from '../src/features/orders/OrderList';

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

describe('OrderList', () => {
  // 注文一覧が正常にレンダリングされることを確認
  it('注文一覧を表示する', async () => {
    render(
      <TestWrapper>
        <OrderList />
      </TestWrapper>
    );

    // ローディング表示の確認
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();

    // データ取得後に注文が表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('CUST-001')).toBeInTheDocument();
      expect(screen.getByText('CUST-002')).toBeInTheDocument();
    });
  });

  // ページタイトルが表示されることを確認
  it('ページタイトルを表示する', async () => {
    render(
      <TestWrapper>
        <OrderList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('注文一覧')).toBeInTheDocument();
    });
  });

  // ステータスフィルターが存在することを確認
  it('ステータスフィルターを表示する', async () => {
    render(
      <TestWrapper>
        <OrderList />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByLabelText('ステータス:')).toBeInTheDocument();
    });
  });
});
