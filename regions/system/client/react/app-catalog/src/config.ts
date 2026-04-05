/// アプリケーション設定の型定義と、ビルド時に注入された設定へのアクセスを提供する
declare const __APP_CONFIG__: AppConfig;

/// アプリケーション全体の設定インターフェース
interface AppConfig {
  app: { name: string; version: string; env: string };
  api: { base_url: string; timeout: number; retry: { max_attempts: number; backoff_ms: number } };
  // BFF設定: ブラウザから見た BFF のベースパス（FE-004 監査対応）
  bff: { base_url: string };
  features: Record<string, boolean>;
}

/// ビルド時に YAML から注入されたアプリケーション設定
export const appConfig: AppConfig = __APP_CONFIG__;
