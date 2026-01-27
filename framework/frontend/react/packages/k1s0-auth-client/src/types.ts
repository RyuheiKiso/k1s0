import { z } from 'zod';

/**
 * JWT Claims のスキーマ
 * バックエンドの k1s0-auth crate の Claims 構造に対応
 */
export const ClaimsSchema = z.object({
  /** ユーザーID (subject) */
  sub: z.string(),
  /** 発行者 (issuer) */
  iss: z.string(),
  /** 対象者 (audience) - 文字列または配列 */
  aud: z.union([z.string(), z.array(z.string())]).optional(),
  /** 有効期限 (expiration time) - Unix timestamp */
  exp: z.number(),
  /** 発行日時 (issued at) - Unix timestamp */
  iat: z.number(),
  /** Not Before - Unix timestamp */
  nbf: z.number().optional(),
  /** JWT ID */
  jti: z.string().optional(),
  /** ロール一覧 */
  roles: z.array(z.string()).default([]),
  /** パーミッション一覧 */
  permissions: z.array(z.string()).default([]),
  /** テナントID */
  tenant_id: z.string().optional(),
  /** スコープ */
  scope: z.string().optional(),
});

export type Claims = z.infer<typeof ClaimsSchema>;

/**
 * OIDC 設定
 */
export interface OIDCConfig {
  /** 認証サーバーの issuer URL */
  issuer: string;
  /** クライアントID */
  clientId: string;
  /** リダイレクトURI */
  redirectUri: string;
  /** スコープ */
  scope?: string;
  /** レスポンスタイプ */
  responseType?: 'code' | 'token' | 'id_token' | 'code token' | 'code id_token';
  /** PKCE を使用するか */
  usePKCE?: boolean;
  /** ログアウト後のリダイレクトURI */
  postLogoutRedirectUri?: string;
}

/**
 * トークンペア
 */
export interface TokenPair {
  /** アクセストークン */
  accessToken: string;
  /** リフレッシュトークン（存在する場合） */
  refreshToken?: string;
  /** IDトークン（OIDC の場合） */
  idToken?: string;
  /** アクセストークンの有効期限（Unix timestamp ms） */
  expiresAt?: number;
  /** トークンのスコープ */
  scope?: string;
  /** トークンタイプ（通常は "Bearer"） */
  tokenType?: string;
}

/**
 * トークン取得の結果
 */
export type TokenResult =
  | { type: 'valid'; token: string; claims: Claims }
  | { type: 'refreshed'; token: string; claims: Claims }
  | { type: 'expired' }
  | { type: 'none' };

/**
 * 認証状態
 */
export type AuthStatus =
  | 'loading'
  | 'authenticated'
  | 'unauthenticated'
  | 'error';

/**
 * 認証コンテキストの状態
 */
export interface AuthState {
  /** 認証状態 */
  status: AuthStatus;
  /** 認証済みの場合のユーザー情報 */
  user?: AuthUser;
  /** エラーの場合のエラー情報 */
  error?: AuthError;
  /** 認証処理中かどうか */
  isLoading: boolean;
  /** 認証済みかどうか */
  isAuthenticated: boolean;
}

/**
 * 認証済みユーザー情報
 */
export interface AuthUser {
  /** ユーザーID */
  id: string;
  /** ロール一覧 */
  roles: string[];
  /** パーミッション一覧 */
  permissions: string[];
  /** テナントID */
  tenantId?: string;
  /** JWT Claims の全体 */
  claims: Claims;
}

/**
 * 認証エラー
 */
export interface AuthError {
  /** エラーコード */
  code: AuthErrorCode;
  /** エラーメッセージ */
  message: string;
  /** 元のエラー */
  cause?: Error;
}

/**
 * 認証エラーコード
 */
export type AuthErrorCode =
  | 'INVALID_TOKEN'
  | 'TOKEN_EXPIRED'
  | 'REFRESH_FAILED'
  | 'NETWORK_ERROR'
  | 'OIDC_ERROR'
  | 'UNAUTHORIZED'
  | 'FORBIDDEN'
  | 'UNKNOWN';

/**
 * トークンストレージのインターフェース
 */
export interface TokenStorage {
  /** トークンを取得 */
  get(): TokenPair | null;
  /** トークンを保存 */
  set(tokens: TokenPair): void;
  /** トークンを削除 */
  clear(): void;
}

/**
 * トークンリフレッシュ関数の型
 */
export type TokenRefresher = (
  refreshToken: string
) => Promise<TokenPair | null>;

/**
 * 認証設定
 */
export interface AuthClientConfig {
  /** OIDC 設定（OIDC フローを使用する場合） */
  oidc?: OIDCConfig;
  /** トークンストレージ（省略時はSessionStorage） */
  storage?: TokenStorage;
  /** トークンリフレッシュ関数（省略時はリフレッシュしない） */
  refreshToken?: TokenRefresher;
  /** 有効期限前のリフレッシュ余裕時間（ms）デフォルト: 60000 (1分) */
  refreshMarginMs?: number;
  /** 自動リフレッシュを有効にするか デフォルト: true */
  autoRefresh?: boolean;
  /** 認証エラー時のコールバック */
  onAuthError?: (error: AuthError) => void;
  /** ログアウト後のコールバック */
  onLogout?: () => void;
  /** 許可する issuer のリスト（検証用） */
  allowedIssuers?: string[];
  /** 許可する audience のリスト（検証用） */
  allowedAudiences?: string[];
}

/**
 * セッション情報
 */
export interface SessionInfo {
  /** セッションID */
  id: string;
  /** セッション開始時刻（Unix timestamp ms） */
  startedAt: number;
  /** 最終アクティブ時刻（Unix timestamp ms） */
  lastActiveAt: number;
  /** セッションの有効期限（Unix timestamp ms） */
  expiresAt?: number;
  /** ユーザーID */
  userId: string;
  /** デバイス情報 */
  deviceInfo?: DeviceInfo;
}

/**
 * デバイス情報
 */
export interface DeviceInfo {
  /** ブラウザ名 */
  browser?: string;
  /** OS名 */
  os?: string;
  /** デバイスタイプ */
  deviceType?: 'desktop' | 'mobile' | 'tablet' | 'unknown';
  /** IPアドレス */
  ipAddress?: string;
}

/**
 * 認証ガードの設定
 */
export interface AuthGuardConfig {
  /** 必要なロール（いずれかを持っていればOK） */
  roles?: string[];
  /** 必要なパーミッション（すべて持っていればOK） */
  permissions?: string[];
  /** 認証されていない場合のリダイレクト先 */
  redirectTo?: string;
  /** 権限不足の場合のリダイレクト先 */
  forbiddenRedirectTo?: string;
  /** カスタム認可チェック関数 */
  authorize?: (user: AuthUser) => boolean | Promise<boolean>;
}
