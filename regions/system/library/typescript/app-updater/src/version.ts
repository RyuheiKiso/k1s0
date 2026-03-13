import type { AppVersionInfo, UpdateType } from './types.js';

function normalizeVersion(version: string): number[] {
  return version
    .split('.')
    .map((segment) => {
      const cleaned = segment.replace(/[^0-9]/g, '');
      return cleaned === '' ? 0 : parseInt(cleaned, 10);
    });
}

export function compareVersions(left: string, right: string): number {
  const leftParts = normalizeVersion(left);
  const rightParts = normalizeVersion(right);
  const length = Math.max(leftParts.length, rightParts.length);

  for (let i = 0; i < length; i++) {
    const leftValue = i < leftParts.length ? leftParts[i] : 0;
    const rightValue = i < rightParts.length ? rightParts[i] : 0;
    if (leftValue !== rightValue) {
      return leftValue < rightValue ? -1 : 1;
    }
  }

  return 0;
}

export function determineUpdateType(
  currentVersion: string,
  versionInfo: AppVersionInfo,
): UpdateType {
  if (
    compareVersions(currentVersion, versionInfo.minimumVersion) < 0 ||
    versionInfo.mandatory
  ) {
    return 'mandatory';
  }

  if (compareVersions(currentVersion, versionInfo.latestVersion) < 0) {
    return 'optional';
  }

  return 'none';
}
