export interface AppUpdaterConfig {
  serverUrl: string;
  appId: string;
  platform?: string;
  arch?: string;
  checkInterval?: number;
  timeout?: number;
}
