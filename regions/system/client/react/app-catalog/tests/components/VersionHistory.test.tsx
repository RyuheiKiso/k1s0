import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { VersionHistory } from '../../src/components/VersionHistory';
import type { AppVersion } from '../../src/api/types';

// DownloadButton 内部の API 呼び出しをスタブ化する
vi.mock('../../src/api/client', () => ({
  fetchDownloadUrl: vi.fn().mockResolvedValue({
    download_url: 'https://example.com/download',
    expires_in: 3600,
    checksum_sha256: 'abc123',
    size_bytes: 1024,
  }),
}));

// テスト用のサンプルバージョンデータ
const mockVersion: AppVersion = {
  id: 'v1',
  app_id: 'app-1',
  version: '1.0.0',
  platform: 'windows',
  arch: 'amd64',
  size_bytes: 1048576,
  checksum_sha256: 'abc123def456',
  release_notes: null,
  mandatory: false,
  published_at: '2024-01-15T00:00:00Z',
};

function renderWithQuery(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

describe('VersionHistory', () => {
  it('バージョンがない場合は空メッセージを表示する', () => {
    renderWithQuery(<VersionHistory versions={[]} appId="app-1" />);
    expect(screen.getByText('バージョンがありません')).toBeInTheDocument();
  });

  it('バージョンがある場合はタイトルを表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('バージョン履歴')).toBeInTheDocument();
  });

  it('バージョン番号を表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('1.0.0')).toBeInTheDocument();
  });

  it('プラットフォームバッジを表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('Windows')).toBeInTheDocument();
  });

  it('アーキテクチャを変換して表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('x64')).toBeInTheDocument();
  });

  it('ファイルサイズを変換して表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('1.0 MB')).toBeInTheDocument();
  });

  it('チェックサムを表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('abc123def456')).toBeInTheDocument();
  });

  it('必須フラグが false の場合は「いいえ」を表示する', () => {
    renderWithQuery(<VersionHistory versions={[mockVersion]} appId="app-1" />);
    expect(screen.getByText('いいえ')).toBeInTheDocument();
  });

  it('必須フラグが true の場合は「はい」を表示する', () => {
    const mandatory = { ...mockVersion, mandatory: true };
    renderWithQuery(<VersionHistory versions={[mandatory]} appId="app-1" />);
    expect(screen.getByText('はい')).toBeInTheDocument();
  });

  it('複数バージョンをすべて表示する', () => {
    const v2: AppVersion = { ...mockVersion, id: 'v2', version: '2.0.0' };
    renderWithQuery(<VersionHistory versions={[mockVersion, v2]} appId="app-1" />);
    expect(screen.getByText('1.0.0')).toBeInTheDocument();
    expect(screen.getByText('2.0.0')).toBeInTheDocument();
  });
});
