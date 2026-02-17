/**
 * OAuth2 Authorization Code + PKCE クライアント
 * Keycloak 対応のクライアント側認証ライブラリ
 */

import { generateCodeVerifier, generateCodeChallenge } from './pkce.js';
import { MemoryTokenStore } from './token-store.js';
import type {
  AuthConfig,
  AuthStateCallback,
  OIDCDiscovery,
  TokenResponse,
  TokenSet,
  TokenStore,
} from './types.js';

export interface AuthClientOptions {
  config: AuthConfig;
  tokenStore?: TokenStore;
  /** fetch 関数の注入（テスト用） */
  fetch?: typeof globalThis.fetch;
  /** リダイレクト関数の注入（テスト用） */
  redirect?: (url: string) => void;
  /** crypto.randomUUID 互換（テスト用） */
  generateState?: () => string;
}

export class AuthClient {
  private readonly config: AuthConfig;
  private readonly tokenStore: TokenStore;
  private readonly fetchFn: typeof globalThis.fetch;
  private readonly redirectFn: (url: string) => void;
  private readonly generateStateFn: () => string;
  private listeners: AuthStateCallback[] = [];
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;
  private discoveryCache: OIDCDiscovery | null = null;

  constructor(options: AuthClientOptions) {
    this.config = options.config;
    this.tokenStore = options.tokenStore ?? new MemoryTokenStore();
    this.fetchFn = options.fetch ?? globalThis.fetch.bind(globalThis);
    this.redirectFn =
      options.redirect ??
      ((url: string) => {
        window.location.href = url;
      });
    this.generateStateFn = options.generateState ?? (() => crypto.randomUUID());
  }

  /**
   * OAuth2 Authorization Code + PKCE フローを開始する。
   * 1. code_verifier を生成
   * 2. code_challenge を計算
   * 3. authorize URL を構築
   * 4. リダイレクト
   */
  async login(): Promise<void> {
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = this.generateStateFn();

    this.tokenStore.setCodeVerifier(codeVerifier);
    this.tokenStore.setState(state);

    const discovery = await this.fetchDiscovery();
    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      scope: this.config.scopes.join(' '),
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
      state,
    });

    this.redirectFn(`${discovery.authorization_endpoint}?${params.toString()}`);
  }

  /**
   * 認可コールバックを処理する。
   * code + code_verifier で token endpoint に POST してトークンを取得・保存する。
   * @param code - 認可コード
   * @param state - state パラメータ（CSRF 対策検証用）
   * @returns トークンセット
   */
  async handleCallback(code: string, state: string): Promise<TokenSet> {
    const savedState = this.tokenStore.getState();
    if (state !== savedState) {
      throw new AuthError('State mismatch');
    }

    const codeVerifier = this.tokenStore.getCodeVerifier();
    if (!codeVerifier) {
      throw new AuthError('Missing PKCE verifier');
    }

    const discovery = await this.fetchDiscovery();
    const resp = await this.fetchFn(discovery.token_endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: this.config.clientId,
        code,
        redirect_uri: this.config.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    if (!resp.ok) {
      throw new AuthError(`Token request failed: ${resp.status}`);
    }

    const data = (await resp.json()) as TokenResponse;
    const tokenSet: TokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };

    this.tokenStore.setTokenSet(tokenSet);
    this.tokenStore.clearCodeVerifier();
    this.tokenStore.clearState();
    this.scheduleRefresh();
    this.notifyListeners(true);

    return tokenSet;
  }

  /**
   * refresh_token で新しいアクセストークンを取得する。
   */
  async refreshToken(): Promise<void> {
    const tokenSet = this.tokenStore.getTokenSet();
    if (!tokenSet?.refreshToken) {
      throw new AuthError('No refresh token');
    }

    const discovery = await this.fetchDiscovery();
    const resp = await this.fetchFn(discovery.token_endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: this.config.clientId,
        refresh_token: tokenSet.refreshToken,
      }),
    });

    if (!resp.ok) {
      // リフレッシュ失敗時はトークンをクリア
      this.tokenStore.clearTokenSet();
      this.notifyListeners(false);
      throw new AuthError(`Token refresh failed: ${resp.status}`);
    }

    const data = (await resp.json()) as TokenResponse;
    const newTokenSet: TokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };

    this.tokenStore.setTokenSet(newTokenSet);
    this.scheduleRefresh();
  }

  /**
   * 有効なアクセストークンを返す。
   * 期限切れ 60 秒前なら自動リフレッシュする。
   */
  async getAccessToken(): Promise<string> {
    const tokenSet = this.tokenStore.getTokenSet();
    if (!tokenSet) {
      throw new AuthError('Not authenticated');
    }

    // 期限切れ 60 秒前ならリフレッシュ
    if (Date.now() >= tokenSet.expiresAt - 60_000) {
      await this.refreshToken();
      const refreshed = this.tokenStore.getTokenSet();
      if (!refreshed) {
        throw new AuthError('Token refresh failed');
      }
      return refreshed.accessToken;
    }

    return tokenSet.accessToken;
  }

  /**
   * 認証状態を返す。
   */
  isAuthenticated(): boolean {
    const tokenSet = this.tokenStore.getTokenSet();
    return tokenSet !== null && Date.now() < tokenSet.expiresAt;
  }

  /**
   * ログアウト処理。
   * トークンを削除し、Keycloak の logout endpoint にリダイレクトする。
   */
  async logout(): Promise<void> {
    const tokenSet = this.tokenStore.getTokenSet();
    this.tokenStore.clearTokenSet();
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
    this.notifyListeners(false);

    // Keycloak の end_session_endpoint にリダイレクト
    if (this.config.logoutUrl || tokenSet?.idToken) {
      try {
        const discovery = await this.fetchDiscovery();
        const params = new URLSearchParams();
        if (tokenSet?.idToken) {
          params.set('id_token_hint', tokenSet.idToken);
        }
        if (this.config.postLogoutRedirectUri) {
          params.set(
            'post_logout_redirect_uri',
            this.config.postLogoutRedirectUri,
          );
          params.set('client_id', this.config.clientId);
        }
        this.redirectFn(
          `${discovery.end_session_endpoint}?${params.toString()}`,
        );
      } catch {
        // Discovery 取得に失敗してもログアウト自体は成功とする
      }
    }
  }

  /**
   * 現在のトークンセットを取得する。
   */
  getTokenSet(): TokenSet | null {
    return this.tokenStore.getTokenSet();
  }

  /**
   * 認証状態の変更を監視するリスナーを登録する。
   * @returns リスナーの登録解除関数
   */
  onAuthStateChange(callback: AuthStateCallback): () => void {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== callback);
    };
  }

  /**
   * リフレッシュをスケジュールする。
   * トークンの有効期限 60 秒前にリフレッシュを実行する。
   */
  private scheduleRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
    }
    const tokenSet = this.tokenStore.getTokenSet();
    if (!tokenSet) return;

    const delay = tokenSet.expiresAt - Date.now() - 60_000;
    if (delay > 0) {
      this.refreshTimer = setTimeout(() => {
        this.refreshToken().catch(() => {
          // リフレッシュ失敗はログアウト状態にする
          this.tokenStore.clearTokenSet();
          this.notifyListeners(false);
        });
      }, delay);
    }
  }

  private notifyListeners(authenticated: boolean): void {
    for (const cb of this.listeners) {
      cb(authenticated);
    }
  }

  private async fetchDiscovery(): Promise<OIDCDiscovery> {
    if (this.discoveryCache) {
      return this.discoveryCache;
    }
    const resp = await this.fetchFn(this.config.discoveryUrl);
    if (!resp.ok) {
      throw new AuthError(`Discovery fetch failed: ${resp.status}`);
    }
    this.discoveryCache = (await resp.json()) as OIDCDiscovery;
    return this.discoveryCache;
  }
}

export class AuthError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AuthError';
  }
}
