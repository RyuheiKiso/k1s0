import type { AppVersion } from '../api/types';

export type PlatformName = AppVersion['platform'];

export const platformLabels: Record<PlatformName, string> = {
  windows: 'Windows',
  linux: 'Linux',
  macos: 'macOS',
};

export function detectClientPlatform(): PlatformName | null {
  if (typeof navigator === 'undefined') {
    return null;
  }

  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes('windows')) {
    return 'windows';
  }
  if (userAgent.includes('mac os') || userAgent.includes('macintosh')) {
    return 'macos';
  }
  if (userAgent.includes('linux')) {
    return 'linux';
  }
  return null;
}

export function formatArch(arch: string): string {
  switch (arch) {
    case 'amd64':
      return 'x64';
    case 'arm64':
      return 'ARM64';
    default:
      return arch;
  }
}

export function formatBytes(bytes: number | null): string {
  if (bytes == null) {
    return '-';
  }
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
