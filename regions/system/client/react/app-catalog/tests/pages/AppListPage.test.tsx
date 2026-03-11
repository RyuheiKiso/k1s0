import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { AppListPage } from '../../src/pages/AppListPage';
import type { App, AppVersion } from '../../src/api/types';

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

const mockVersions: Record<string, AppVersion[]> = {
  'app-1': [
    {
      id: 'ver-1',
      app_id: 'app-1',
      version: '1.2.0',
      platform: 'windows',
      arch: 'amd64',
      size_bytes: 1024,
      checksum_sha256: 'checksum-a',
      release_notes: '改善',
      mandatory: false,
      published_at: '2024-01-03T00:00:00Z',
    },
  ],
  'app-2': [
    {
      id: 'ver-2',
      app_id: 'app-2',
      version: '2.0.0',
      platform: 'macos',
      arch: 'arm64',
      size_bytes: 2048,
      checksum_sha256: 'checksum-b',
      release_notes: '更新',
      mandatory: false,
      published_at: '2024-01-04T00:00:00Z',
    },
  ],
};

const server = setupServer(
  http.get('/api/v1/apps', () => {
    return HttpResponse.json({ apps: mockApps });
  }),
  http.get('/api/v1/apps/:appId/versions', ({ params }) => {
    return HttpResponse.json({ versions: mockVersions[String(params.appId)] ?? [] });
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
