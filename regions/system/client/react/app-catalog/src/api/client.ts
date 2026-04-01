import { createApiClient } from '../lib/systemClient';
import { appConfig } from '../config';
import type {
  App,
  AppListParams,
  AppListResponse,
  AppVersion,
  DownloadStats,
  DownloadUrlResponse,
  VersionListResponse,
} from './types';

// 未認証時のナビゲーション関数（テスト時にモック差し替え可能）
// window.location.href を直接参照しないことでテスト環境での副作用を防ぐ
let navigateToLogin = () => { window.location.href = '/auth/login'; };
export const setNavigateToLogin = (fn: () => void) => { navigateToLogin = fn; };

// BFF プロキシ経由でリクエストを送信する（L-11 監査対応: ハードコードを廃止し appConfig から取得する）
const api = createApiClient({
  baseURL: appConfig.api.base_url,
  // 未認証時は navigateToLogin を経由することでテスト時のモック差し替えを可能にする
  onUnauthorized: () => navigateToLogin(),
});

export async function fetchApps(params?: AppListParams): Promise<App[]> {
  const { data } = await api.get<AppListResponse>('/apps', { params });
  return data.apps;
}

export async function fetchApp(appId: string): Promise<App> {
  const { data } = await api.get<App>(`/apps/${appId}`);
  return data;
}

export async function fetchAppVersions(appId: string): Promise<AppVersion[]> {
  const { data } = await api.get<VersionListResponse>(`/apps/${appId}/versions`);
  return data.versions;
}

export async function fetchAppDetail(appId: string): Promise<{ app: App; versions: AppVersion[] }> {
  const [app, versions] = await Promise.all([fetchApp(appId), fetchAppVersions(appId)]);
  return { app, versions };
}

export async function fetchDownloadUrl(
  appId: string,
  version: string,
  platform?: AppVersion['platform'],
  arch?: string,
): Promise<DownloadUrlResponse> {
  const { data } = await api.get<DownloadUrlResponse>(
    `/apps/${appId}/versions/${encodeURIComponent(version)}/download`,
    {
      params: {
        platform,
        arch,
      },
    },
  );
  return data;
}

export async function fetchDownloadStats(appId: string): Promise<DownloadStats> {
  const { data } = await api.get<DownloadStats>(`/apps/${appId}/stats`);
  return data;
}

export { api };
