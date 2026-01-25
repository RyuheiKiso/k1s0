import type {
  TokenPair,
  TokenResult,
  TokenStorage,
  TokenRefresher,
} from './types.js';

const STORAGE_KEY = 'k1s0_auth_tokens';

/**
 * SessionStorageを使用するデフォルトのトークンストレージ
 */
export class SessionTokenStorage implements TokenStorage {
  get(): TokenPair | null {
    if (typeof window === 'undefined') return null;
    try {
      const stored = sessionStorage.getItem(STORAGE_KEY);
      if (!stored) return null;
      return JSON.parse(stored) as TokenPair;
    } catch {
      return null;
    }
  }

  set(tokens: TokenPair): void {
    if (typeof window === 'undefined') return;
    try {
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(tokens));
    } catch {
      // ストレージ書き込み失敗時は無視
    }
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    try {
      sessionStorage.removeItem(STORAGE_KEY);
    } catch {
      // ストレージ削除失敗時は無視
    }
  }
}

/**
 * LocalStorageを使用するトークンストレージ（永続化が必要な場合）
 */
export class LocalTokenStorage implements TokenStorage {
  get(): TokenPair | null {
    if (typeof window === 'undefined') return null;
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (!stored) return null;
      return JSON.parse(stored) as TokenPair;
    } catch {
      return null;
    }
  }

  set(tokens: TokenPair): void {
    if (typeof window === 'undefined') return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(tokens));
    } catch {
      // ストレージ書き込み失敗時は無視
    }
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ストレージ削除失敗時は無視
    }
  }
}

/**
 * トークン管理クラス
 * - トークンの保存/取得/削除
 * - 有効期限の確認
 * - 自動リフレッシュ（設定時）
 */
export class TokenManager {
  private storage: TokenStorage;
  private refresher: TokenRefresher | undefined;
  private refreshMarginMs: number;
  private refreshPromise: Promise<TokenPair | null> | null = null;

  constructor(options?: {
    storage?: TokenStorage;
    refreshToken?: TokenRefresher;
    refreshMarginMs?: number;
  }) {
    this.storage = options?.storage ?? new SessionTokenStorage();
    this.refresher = options?.refreshToken;
    this.refreshMarginMs = options?.refreshMarginMs ?? 60_000; // デフォルト1分
  }

  /**
   * トークンを保存
   */
  setTokens(tokens: TokenPair): void {
    this.storage.set(tokens);
  }

  /**
   * トークンをクリア（ログアウト時）
   */
  clearTokens(): void {
    this.storage.clear();
    this.refreshPromise = null;
  }

  /**
   * 現在のトークンペアを取得
   */
  getTokens(): TokenPair | null {
    return this.storage.get();
  }

  /**
   * トークンが有効期限内かどうかを確認
   */
  isTokenValid(tokens: TokenPair): boolean {
    if (!tokens.expiresAt) {
      // 有効期限が設定されていない場合は有効とみなす
      return true;
    }
    return Date.now() < tokens.expiresAt - this.refreshMarginMs;
  }

  /**
   * トークンがリフレッシュ必要かどうかを確認
   */
  needsRefresh(tokens: TokenPair): boolean {
    if (!tokens.expiresAt || !tokens.refreshToken) {
      return false;
    }
    // 有効期限前のマージン時間に入ったらリフレッシュ
    return Date.now() >= tokens.expiresAt - this.refreshMarginMs;
  }

  /**
   * 有効なアクセストークンを取得（必要に応じてリフレッシュ）
   */
  async getValidToken(): Promise<TokenResult> {
    const tokens = this.storage.get();

    if (!tokens) {
      return { type: 'none' };
    }

    // トークンが有効期限内ならそのまま返す
    if (this.isTokenValid(tokens)) {
      return { type: 'valid', token: tokens.accessToken };
    }

    // リフレッシュが必要かつ可能な場合
    if (tokens.refreshToken && this.refresher) {
      const refreshed = await this.tryRefresh(tokens.refreshToken);
      if (refreshed) {
        return { type: 'refreshed', token: refreshed.accessToken };
      }
    }

    // リフレッシュ失敗または不可能
    return { type: 'expired' };
  }

  /**
   * トークンリフレッシュを試行（重複リクエスト防止付き）
   */
  private async tryRefresh(refreshToken: string): Promise<TokenPair | null> {
    if (!this.refresher) {
      return null;
    }

    // 既にリフレッシュ中の場合は同じPromiseを返す
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    this.refreshPromise = this.refresher(refreshToken)
      .then((newTokens) => {
        if (newTokens) {
          this.storage.set(newTokens);
        } else {
          this.storage.clear();
        }
        return newTokens;
      })
      .catch(() => {
        this.storage.clear();
        return null;
      })
      .finally(() => {
        this.refreshPromise = null;
      });

    return this.refreshPromise;
  }
}
