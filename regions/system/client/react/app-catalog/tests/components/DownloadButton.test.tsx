import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DownloadButton } from '../../src/components/DownloadButton';
import { fetchDownloadUrl } from '../../src/api/client';

vi.mock('../../src/api/client', () => ({
  fetchDownloadUrl: vi.fn().mockResolvedValue({
    download_url: 'https://example.com/download',
    expires_in: 3600,
    checksum_sha256: 'checksum',
    size_bytes: 100,
  }),
}));

function renderWithQuery(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

describe('DownloadButton', () => {
  it('デフォルトのラベルを表示する', () => {
    renderWithQuery(<DownloadButton appId="app-1" version="1.0.0" />);
    expect(screen.getByTestId('download-button')).toHaveTextContent('ダウンロード');
  });

  it('カスタムラベルを表示する', () => {
    renderWithQuery(<DownloadButton appId="app-1" version="1.0.0" label="v1.0.0 をダウンロード" />);
    expect(screen.getByTestId('download-button')).toHaveTextContent('v1.0.0 をダウンロード');
  });

  it('クリックでダウンロードが発火する', async () => {
    const openSpy = vi.mocked(window.open);
    renderWithQuery(<DownloadButton appId="app-1" version="1.0.0" platform="windows" arch="amd64" />);
    const button = screen.getByTestId('download-button');
    fireEvent.click(button);

    await waitFor(() => {
      expect(fetchDownloadUrl).toHaveBeenCalledWith('app-1', '1.0.0', 'windows', 'amd64');
      expect(openSpy).toHaveBeenCalledWith(
        'https://example.com/download',
        '_blank',
        'noopener,noreferrer',
      );
    });
  });
});
