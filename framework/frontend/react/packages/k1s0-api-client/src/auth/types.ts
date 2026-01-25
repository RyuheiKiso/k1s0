/**
 * 認証トークンのペア
 */
export interface TokenPair {
  /** アクセストークン */
  accessToken: string;
  /** リフレッシュトークン（存在する場合） */
  refreshToken?: string;
  /** アクセストークンの有効期限（Unix timestamp ms） */
  expiresAt?: number;
}

/**
 * トークン取得の結果
 */
export type TokenResult =
  | { type: 'valid'; token: string }
  | { type: 'refreshed'; token: string }
  | { type: 'expired' }
  | { type: 'none' };

/**
 * 認証状態
 */
export type AuthState =
  | { status: 'loading' }
  | { status: 'authenticated'; accessToken: string }
  | { status: 'unauthenticated' };

/**
 * トークンストレージのインターフェース
 * デフォルト実装はSessionStorage/LocalStorageを使用するが、
 * アプリ側で差し替え可能
 */
export interface TokenStorage {
  get(): TokenPair | null;
  set(tokens: TokenPair): void;
  clear(): void;
}

/**
 * トークンリフレッシュ関数の型
 * アプリ側で認証サービスへの接続を実装
 */
export type TokenRefresher = (
  refreshToken: string
) => Promise<TokenPair | null>;

/**
 * 認証コンテキストの設定
 */
export interface AuthConfig {
  /** トークンストレージ（省略時はSessionStorage） */
  storage?: TokenStorage;
  /** トークンリフレッシュ関数（省略時はリフレッシュしない） */
  refreshToken?: TokenRefresher;
  /** 有効期限前のリフレッシュ余裕時間（ms）デフォルト: 60000 (1分) */
  refreshMarginMs?: number;
  /** 認証エラー時のコールバック（リダイレクト等） */
  onAuthError?: () => void;
}
