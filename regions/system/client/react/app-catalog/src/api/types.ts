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
}

export interface AppListParams {
  search?: string;
  category?: string;
}

export interface AppDetailResponse {
  app: App;
  versions: AppVersion[];
}
