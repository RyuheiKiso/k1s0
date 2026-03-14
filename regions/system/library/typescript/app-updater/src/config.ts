/** アプリアップデーターの設定 */
export interface AppUpdaterConfig {
  /** App Registry サーバーの URL */
  serverUrl: string;
  /** アプリケーション ID */
  appId: string;
  /** プラットフォーム（例: "linux", "darwin", "windows"） */
  platform?: string;
  /** CPU アーキテクチャ（例: "amd64", "arm64"） */
  arch?: string;
  /** アップデート確認の間隔（ミリ秒） */
  checkInterval?: number;
  /** HTTP リクエストのタイムアウト（ミリ秒） */
  timeout?: number;
}
