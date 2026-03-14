import type { AppVersionInfo, UpdateType } from './types.js';

/**
 * バージョン文字列を数値の配列に正規化する
 *
 * プレリリースサフィックス（例: "-beta"）は無視して数値のみを抽出する。
 * 例: "1.2.3-beta" → [1, 2, 3]
 */
function normalizeVersion(version: string): number[] {
  return version
    .split('.')
    .map((segment) => {
      const cleaned = segment.replace(/[^0-9]/g, '');
      return cleaned === '' ? 0 : parseInt(cleaned, 10);
    });
}

/**
 * 2つのバージョン文字列を比較する
 *
 * @returns 負の値: left < right、0: 同一、正の値: left > right
 */
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

/**
 * 現在のバージョンとサーバーのバージョン情報からアップデート種別を判定する
 *
 * - 現在バージョンが最低バージョンを下回る、または `mandatory` が true → `'mandatory'`
 * - 現在バージョンが最新バージョンを下回る → `'optional'`
 * - それ以外 → `'none'`
 */
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
