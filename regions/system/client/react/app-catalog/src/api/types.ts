export interface App {
  id: string;
  name: string;
  description: string | null;
  category: string;
  icon_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface AppVersion {
  id: string;
  app_id: string;
  version: string;
  platform: 'windows' | 'linux' | 'macos';
  arch: string;
  size_bytes: number | null;
  checksum_sha256: string;
  release_notes: string | null;
  mandatory: boolean;
  published_at: string;
  download_url?: string;
}

export interface DownloadStats {
  total_downloads: number;
  version_downloads: number;
  latest_version?: string | null;
}

export interface DownloadUrlResponse {
  download_url: string;
  expires_in: number;
  checksum_sha256: string;
  size_bytes: number | null;
}

export interface AppListParams {
  search?: string;
  category?: string;
}

export interface AppListResponse {
  apps: App[];
}

export interface VersionListResponse {
  versions: AppVersion[];
}
