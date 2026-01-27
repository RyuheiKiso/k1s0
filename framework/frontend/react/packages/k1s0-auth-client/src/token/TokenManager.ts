import type {
  TokenPair,
  TokenResult,
  TokenStorage,
  TokenRefresher,
  AuthError,
  Claims,
} from '../types.js';
import { SessionTokenStorage } from './storage.js';
import {
  decodeToken,
  isTokenValid,
  needsRefresh,
  getTimeUntilExpiry,
  claimsToUser,
} from './decoder.js';

/**
 * TokenManager の設定
 */
export interface TokenManagerOptions {
  /** トークンストレージ */
  storage?: TokenStorage;
  /** トークンリフレッシュ関数 */
  refreshToken?: TokenRefresher;
  /** 有効期限前のリフレッシュ余裕時間（ms） */
  refreshMarginMs?: number;
  /** 自動リフレッシュを有効にするか */
  autoRefresh?: boolean;
  /** リフレッシュ失敗時のコールバック */
  onRefreshError?: (error: AuthError) => void;
}

/**
 * トークン管理クラス
 *
 * - トークンの保存/取得/削除
 * - JWT のデコードと検証
 * - 有効期限の確認
 * - 自動リフレッシュ（設定時）
 * - 重複リフレッシュの防止
 */
export class TokenManager {
  private storage: TokenStorage;
  private refresher: TokenRefresher | undefined;
  private refreshMarginMs: number;
  private autoRefresh: boolean;
  private onRefreshError: ((error: AuthError) => void) | undefined;

  // リフレッシュの重複防止用
  private refreshPromise: Promise<TokenPair | null> | null = null;

  // 自動リフレッシュ用タイマー
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  // リフレッシュイベントのリスナー
  private refreshListeners: Set<(tokens: TokenPair | null) => void> = new Set();

  constructor(options?: TokenManagerOptions) {
    this.storage = options?.storage ?? new SessionTokenStorage();
    this.refresher = options?.refreshToken;
    this.refreshMarginMs = options?.refreshMarginMs ?? 60_000; // デフォルト1分
    this.autoRefresh = options?.autoRefresh ?? true;
    this.onRefreshError = options?.onRefreshError;
  }

  /**
   * トークンを保存
   */
  setTokens(tokens: TokenPair): void {
    this.storage.set(tokens);

    // 自動リフレッシュのスケジューリング
    if (this.autoRefresh && tokens.refreshToken) {
      this.scheduleRefresh(tokens);
    }
  }

  /**
   * トークンをクリア（ログアウト時）
   */
  clearTokens(): void {
    this.storage.clear();
    this.refreshPromise = null;
    this.cancelScheduledRefresh();
  }

  /**
   * 現在のトークンペアを取得
   */
  getTokens(): TokenPair | null {
    return this.storage.get();
  }

  /**
   * 現在のアクセストークンの Claims を取得
   */
  getClaims(): Claims | null {
    const tokens = this.storage.get();
    if (!tokens) return null;

    const result = decodeToken(tokens.accessToken);
    if (!result.success) return null;

    return result.claims;
  }

  /**
   * トークンが有効期限内かどうかを確認
   */
  isValid(): boolean {
    const tokens = this.storage.get();
    if (!tokens) return false;

    const result = decodeToken(tokens.accessToken);
    if (!result.success) return false;

    return isTokenValid(result.claims, 0);
  }

  /**
   * トークンがリフレッシュ可能かどうかを確認
   */
  canRefresh(): boolean {
    const tokens = this.storage.get();
    return !!(tokens?.refreshToken && this.refresher);
  }

  /**
   * 有効なアクセストークンを取得（必要に応じてリフレッシュ）
   */
  async getValidToken(): Promise<TokenResult> {
    const tokens = this.storage.get();

    if (!tokens) {
      return { type: 'none' };
    }

    const result = decodeToken(tokens.accessToken);
    if (!result.success) {
      this.clearTokens();
      return { type: 'none' };
    }

    const { claims } = result;

    // トークンが有効期限内ならそのまま返す
    if (isTokenValid(claims, this.refreshMarginMs)) {
      return {
        type: 'valid',
        token: tokens.accessToken,
        claims,
      };
    }

    // リフレッシュが必要かつ可能な場合
    if (tokens.refreshToken && this.refresher) {
      const refreshed = await this.tryRefresh(tokens.refreshToken);
      if (refreshed) {
        const newResult = decodeToken(refreshed.accessToken);
        if (newResult.success) {
          return {
            type: 'refreshed',
            token: refreshed.accessToken,
            claims: newResult.claims,
          };
        }
      }
    }

    // リフレッシュ失敗または不可能
    return { type: 'expired' };
  }

  /**
   * トークンリフレッシュを強制実行
   */
  async forceRefresh(): Promise<TokenPair | null> {
    const tokens = this.storage.get();
    if (!tokens?.refreshToken || !this.refresher) {
      return null;
    }
    return this.tryRefresh(tokens.refreshToken);
  }

  /**
   * リフレッシュイベントのリスナーを追加
   */
  onRefresh(listener: (tokens: TokenPair | null) => void): () => void {
    this.refreshListeners.add(listener);
    return () => {
      this.refreshListeners.delete(listener);
    };
  }

  /**
   * リソースの解放
   */
  dispose(): void {
    this.cancelScheduledRefresh();
    this.refreshListeners.clear();
    this.refreshPromise = null;
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

    this.refreshPromise = this.executeRefresh(refreshToken);
    return this.refreshPromise;
  }

  private async executeRefresh(refreshToken: string): Promise<TokenPair | null> {
    try {
      const newTokens = await this.refresher!(refreshToken);

      if (newTokens) {
        this.storage.set(newTokens);
        this.notifyRefreshListeners(newTokens);

        // 次回の自動リフレッシュをスケジュール
        if (this.autoRefresh && newTokens.refreshToken) {
          this.scheduleRefresh(newTokens);
        }
      } else {
        this.storage.clear();
        this.notifyRefreshListeners(null);
        this.onRefreshError?.({
          code: 'REFRESH_FAILED',
          message: 'Token refresh returned null',
        });
      }

      return newTokens;
    } catch (err) {
      this.storage.clear();
      this.notifyRefreshListeners(null);

      const error: AuthError = {
        code: 'REFRESH_FAILED',
        message: err instanceof Error ? err.message : 'Token refresh failed',
        cause: err instanceof Error ? err : undefined,
      };
      this.onRefreshError?.(error);

      return null;
    } finally {
      this.refreshPromise = null;
    }
  }

  /**
   * 自動リフレッシュをスケジュール
   */
  private scheduleRefresh(tokens: TokenPair): void {
    this.cancelScheduledRefresh();

    const result = decodeToken(tokens.accessToken);
    if (!result.success) return;

    const timeUntilExpiry = getTimeUntilExpiry(result.claims);
    const refreshIn = Math.max(
      timeUntilExpiry - this.refreshMarginMs,
      1000 // 最低1秒後
    );

    if (refreshIn > 0 && tokens.refreshToken) {
      this.refreshTimer = setTimeout(() => {
        this.tryRefresh(tokens.refreshToken!);
      }, refreshIn);
    }
  }

  /**
   * スケジュールされた自動リフレッシュをキャンセル
   */
  private cancelScheduledRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  /**
   * リフレッシュリスナーに通知
   */
  private notifyRefreshListeners(tokens: TokenPair | null): void {
    for (const listener of this.refreshListeners) {
      try {
        listener(tokens);
      } catch {
        // リスナーエラーは無視
      }
    }
  }
}
