import type { AppVersionInfo, UpdateCheckResult } from './types.js';
import type { AppUpdaterConfig } from './config.js';
import { InvalidConfigError } from './error.js';
import { compareVersions, determineUpdateType } from './version.js';

/** アプリアップデーター共通インターフェース */
export interface AppUpdater {
  /** App Registry サーバーから最新バージョン情報を取得する */
  fetchVersionInfo(): Promise<AppVersionInfo>;
  /** 現在のバージョンとサーバーの情報を比較してアップデート確認結果を返す */
  checkForUpdate(): Promise<UpdateCheckResult>;
}

/** App Registry サーバーと通信する本番用アップデーター */
export class AppRegistryAppUpdater implements AppUpdater {
  private readonly config: AppUpdaterConfig;
  /** アプリケーションの現在バージョン */
  private readonly currentVersion: string;

  /**
   * @throws {InvalidConfigError} serverUrl または appId が空の場合
   */
  constructor(config: AppUpdaterConfig, currentVersion: string) {
    if (!config.serverUrl.trim()) {
      throw new InvalidConfigError('serverUrl must not be empty.');
    }
    if (!config.appId.trim()) {
      throw new InvalidConfigError('appId must not be empty.');
    }
    this.config = config;
    this.currentVersion = currentVersion;
  }

  async fetchVersionInfo(): Promise<AppVersionInfo> {
    // エンドポイント URL を組み立て、platform / arch クエリパラメータを付加する
    const url = new URL(
      `/api/v1/apps/${this.config.appId}/versions/latest`,
      this.config.serverUrl,
    );
    if (this.config.platform) {
      url.searchParams.set('platform', this.config.platform);
    }
    if (this.config.arch) {
      url.searchParams.set('arch', this.config.arch);
    }

    const response = await fetch(url.toString(), {
      signal: this.config.timeout
        ? AbortSignal.timeout(this.config.timeout)
        : undefined,
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch version info: ${response.status}`);
    }

    // スネークケースのレスポンスをキャメルケースに変換する
    const data = (await response.json()) as Record<string, unknown>;
    return {
      latestVersion: data['latest_version'] as string,
      minimumVersion: data['minimum_version'] as string,
      mandatory: data['mandatory'] as boolean,
      releaseNotes: data['release_notes'] as string | undefined,
      publishedAt: data['published_at']
        ? new Date(data['published_at'] as string)
        : undefined,
    };
  }

  async checkForUpdate(): Promise<UpdateCheckResult> {
    const versionInfo = await this.fetchVersionInfo();
    const updateType = determineUpdateType(this.currentVersion, versionInfo);

    return {
      currentVersion: this.currentVersion,
      latestVersion: versionInfo.latestVersion,
      minimumVersion: versionInfo.minimumVersion,
      updateType,
      releaseNotes: versionInfo.releaseNotes,
    };
  }
}

/**
 * テスト・スタブ用のインメモリアップデーター
 *
 * サーバーへの HTTP 通信を行わず、内部に保持したバージョン情報を返す。
 * ユニットテストや依存性注入での差し替えに使用する。
 */
export class InMemoryAppUpdater implements AppUpdater {
  private versionInfo: AppVersionInfo;
  /** アプリケーションの現在バージョン */
  private currentVersion: string;

  constructor(versionInfo: AppVersionInfo, currentVersion: string) {
    this.versionInfo = versionInfo;
    this.currentVersion = currentVersion;
  }

  async fetchVersionInfo(): Promise<AppVersionInfo> {
    return this.versionInfo;
  }

  async checkForUpdate(): Promise<UpdateCheckResult> {
    const updateType = determineUpdateType(this.currentVersion, this.versionInfo);

    return {
      currentVersion: this.currentVersion,
      latestVersion: this.versionInfo.latestVersion,
      minimumVersion: this.versionInfo.minimumVersion,
      updateType,
      releaseNotes: this.versionInfo.releaseNotes,
    };
  }

  /** バージョン情報を更新する（テスト用） */
  setVersionInfo(info: AppVersionInfo): void {
    this.versionInfo = info;
  }

  /** 現在バージョンを更新する（テスト用） */
  setCurrentVersion(version: string): void {
    this.currentVersion = version;
  }
}
