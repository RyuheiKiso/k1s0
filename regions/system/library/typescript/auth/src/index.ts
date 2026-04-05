export { AuthClient, AuthError } from './auth-client.js';
export type { AuthClientOptions } from './auth-client.js';
export { DeviceAuthClient, DeviceFlowError } from './device-flow.js';
export type {
  DeviceAuthClientOptions,
  DeviceCodeCallback,
  DeviceCodeResponse,
  DeviceTokenResponse,
} from './device-flow.js';
export { generateCodeVerifier, generateCodeChallenge, base64UrlEncode } from './pkce.js';
export { MemoryTokenStore } from './token-store.js';
// @deprecated DevLocalStorageTokenStore は開発・テスト専用です。本番環境での使用は禁止されています。
// XSS 攻撃に脆弱なため、本番環境では SecureTokenStore を使用してください。
// 将来のバージョンでは dev サブパス（@k1s0/auth/dev）に移動予定です。
export { DevLocalStorageTokenStore } from './token-store.js';
// H-007 監査対応: 本番環境用の BFF パターンを使用した安全なトークンストア。
// httpOnly Cookie 経由でトークンを管理するため、XSS によるトークン窃取を防止する。
// M-010 監査対応: SecureTokenStoreOptions を公開し、呼び出し元がトークン失敗コールバックを設定できるようにする。
export { SecureTokenStore } from './token-store.js';
export type { SecureTokenStoreOptions } from './token-store.js';
export type {
  AuthConfig,
  TokenResponse,
  TokenSet,
  Claims,
  OIDCDiscovery,
  AuthStateCallback,
  TokenStore,
} from './types.js';
