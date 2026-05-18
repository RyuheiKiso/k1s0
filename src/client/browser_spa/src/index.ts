// Browser SPA SDK のエントリポイント
// WebCrypto AES-GCM による暗号化ストレージと HttpOnly cookie を提供する

// SDK バージョンの定義
export const SDK_VERSION = '1.0.0';

// SDK クラスの識別子
export const SDK_CLASS = 'browser_spa';

// 初期化関数の型定義
export type SdkConfig = {
  // tier1 gateway の base URL
  gatewayUrl: string;
  // tenant_id は不要 (AuthContext から自動注入される)
};

// Browser SPA SDK の初期化関数
export function initSdk(_config: SdkConfig): void {
  // TODO: 実際の SDK 初期化ロジックを Stage 5 後半で実装する
  // WebCrypto / Service Worker の設定を行う
}
