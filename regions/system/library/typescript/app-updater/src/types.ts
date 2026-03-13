export type UpdateType = 'none' | 'optional' | 'mandatory';

export interface AppVersionInfo {
  latestVersion: string;
  minimumVersion: string;
  mandatory: boolean;
  releaseNotes?: string;
  publishedAt?: Date;
}

export interface UpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  minimumVersion: string;
  updateType: UpdateType;
  releaseNotes?: string;
}

export interface DownloadArtifactInfo {
  url: string;
  checksum: string;
  size?: number;
  expiresAt?: Date;
}
