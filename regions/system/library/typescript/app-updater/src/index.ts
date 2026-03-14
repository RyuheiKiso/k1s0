// 型定義のエクスポート
export type {
  UpdateType,
  AppVersionInfo,
  UpdateCheckResult,
  DownloadArtifactInfo,
} from './types.js';
export type { AppUpdaterConfig } from './config.js';

// エラークラスのエクスポート
export {
  AppUpdaterError,
  ConnectionError,
  InvalidConfigError,
  ParseError,
  UnauthorizedError,
  AppNotFoundError,
  VersionNotFoundError,
  ChecksumError,
} from './error.js';

// ユーティリティ関数のエクスポート
export { compareVersions, determineUpdateType } from './version.js';
export { ChecksumVerifier } from './checksum.js';

// クライアントクラスのエクスポート
export type { AppUpdater } from './client.js';
export { AppRegistryAppUpdater, InMemoryAppUpdater } from './client.js';
