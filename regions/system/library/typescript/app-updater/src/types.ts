/** アップデートの種別 */
export type UpdateType = 'none' | 'optional' | 'mandatory';

/** App Registry サーバーから取得するバージョン情報 */
export interface AppVersionInfo {
  /** 最新バージョン */
  latestVersion: string;
  /** 最低動作バージョン（これを下回る場合は強制アップデート） */
  minimumVersion: string;
  /** 強制アップデートフラグ */
  mandatory: boolean;
  /** リリースノート */
  releaseNotes?: string;
  /** リリース日時 */
  publishedAt?: Date;
}

/** アップデート確認結果 */
export interface UpdateCheckResult {
  /** 現在のバージョン */
  currentVersion: string;
  /** 最新バージョン */
  latestVersion: string;
  /** 最低動作バージョン */
  minimumVersion: string;
  /** アップデートの種別 */
  updateType: UpdateType;
  /** リリースノート */
  releaseNotes?: string;
}

/** ダウンロードアーティファクト情報 */
export interface DownloadArtifactInfo {
  /** ダウンロード URL */
  url: string;
  /** SHA-256 チェックサム（16進数文字列） */
  checksum: string;
  /** ファイルサイズ（バイト） */
  size?: number;
  /** URL の有効期限 */
  expiresAt?: Date;
}
