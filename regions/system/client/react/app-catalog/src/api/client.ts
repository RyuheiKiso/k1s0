import { createApiClient } from 'system-client';
import type { App, AppVersion, AppListParams, AppDetailResponse, DownloadStats } from './types';

const api = createApiClient({ baseURL: '/api' });

export async function fetchApps(params?: AppListParams): Promise<App[]> {
  const { data } = await api.get<App[]>('/apps', { params });
  return data;
}

export async function fetchAppDetail(appId: string): Promise<AppDetailResponse> {
  const { data } = await api.get<AppDetailResponse>(`/apps/${appId}`);
  return data;
}

export async function fetchAppVersions(appId: string): Promise<AppVersion[]> {
  const { data } = await api.get<AppVersion[]>(`/apps/${appId}/versions`);
  return data;
}

export async function fetchDownloadUrl(appId: string, versionId: string): Promise<string> {
  const { data } = await api.get<{ url: string }>(`/apps/${appId}/versions/${versionId}/download`);
  return data.url;
}

export async function fetchDownloadStats(appId: string): Promise<DownloadStats> {
  const { data } = await api.get<DownloadStats>(`/apps/${appId}/stats`);
  return data;
}

export { api };
