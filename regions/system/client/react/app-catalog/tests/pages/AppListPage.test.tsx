import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { AppListPage } from '../../src/pages/AppListPage';
import type { App } from '../../src/api/types';

const mockApps: App[] = [
  {
    id: 'app-1',
    name: 'アプリA',
    description: '最初のアプリ',
    category: 'ツール',
    icon_url: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: 'app-2',
    name: 'アプリB',
    description: '2番目のアプリ',
    category: '開発',
    icon_url: null,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
];

const server = setupServer(
  http.get('/api/apps', () => {
    return HttpResponse.json(mockApps);
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <AppListPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe('AppListPage', () => {
  it('アプリ一覧を表示する', async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText('アプリA')).toBeInTheDocument();
    });

    expect(screen.getByText('アプリB')).toBeInTheDocument();
  });

  it('ページタイトルを表示する', async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getByText('アプリカタログ')).toBeInTheDocument();
    });
  });

  it('読み込み中の表示をする', () => {
    renderPage();
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
  });
});
