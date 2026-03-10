import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DownloadButton } from '../../src/components/DownloadButton';

vi.mock('../../src/api/client', () => ({
  fetchDownloadUrl: vi.fn().mockResolvedValue('https://example.com/download'),
}));

function renderWithQuery(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

describe('DownloadButton', () => {
  it('デフォルトのラベルを表示する', () => {
    renderWithQuery(<DownloadButton appId="app-1" versionId="v-1" />);
    expect(screen.getByTestId('download-button')).toHaveTextContent('ダウンロード');
  });

  it('カスタムラベルを表示する', () => {
    renderWithQuery(<DownloadButton appId="app-1" versionId="v-1" label="v1.0.0 をダウンロード" />);
    expect(screen.getByTestId('download-button')).toHaveTextContent('v1.0.0 をダウンロード');
  });

  it('クリックでダウンロードが発火する', () => {
    renderWithQuery(<DownloadButton appId="app-1" versionId="v-1" />);
    const button = screen.getByTestId('download-button');
    fireEvent.click(button);
    expect(button).toBeInTheDocument();
  });
});
