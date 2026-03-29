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

// BFF プロキシ経由でリクエストを送信する（L-11 監査対応: ハードコードを廃止し appConfig から取得する）
const api = createApiClient({
  baseURL: appConfig.api.base_url,
  onUnauthorized: () => { window.location.href = '/auth/login'; },
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
