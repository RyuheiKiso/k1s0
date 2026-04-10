/**
 * トークン保存ストア
 * メモリストア、localStorage ストア（開発専用）、BFF 経由の安全なストアの 3 種類を提供する。
 */

import type { TokenSet, TokenStore } from './types.js';

/**
 * メモリベースのトークンストア。
 * テスト用、または SSR 環境で使用する。
 */
export class MemoryTokenStore implements TokenStore {
  private tokenSet: TokenSet | null = null;
  private codeVerifier: string | null = null;
  private state: string | null = null;

  getTokenSet(): TokenSet | null {
    return this.tokenSet;
  }

  // HIGH-FE-001 対応: setTokenSet を async Promise<void> に変更する（インターフェース準拠）
  async setTokenSet(tokenSet: TokenSet): Promise<void> {
    this.tokenSet = tokenSet;
  }

  clearTokenSet(): void {
    this.tokenSet = null;
  }

  getCodeVerifier(): string | null {
    return this.codeVerifier;
  }

  setCodeVerifier(verifier: string): void {
    this.codeVerifier = verifier;
  }

  clearCodeVerifier(): void {
    this.codeVerifier = null;
  }

  getState(): string | null {
    return this.state;
  }

  setState(state: string): void {
    this.state = state;
  }

  clearState(): void {
    this.state = null;
  }
}

/**
 * localStorage ベースのトークンストア（開発専用）。
 * ブラウザ環境で使用する。
 * PKCE verifier と state は sessionStorage に保存する（タブ間の分離のため）。
 *
 * @deprecated 開発・テスト専用。本番環境では使用しないこと。
 * XSS 攻撃に脆弱です。本番環境では `SecureTokenStore` を使用してください。
 * @see SecureTokenStore
 *
 * @security セキュリティ警告
 *
 * **この実装は開発・テスト用途のみを想定しています。本番環境での使用は禁止です。**
 * クラス名の "Dev" プレフィックスはこの制約を明示するためのものです。
 *
 * localStorage はXSS（クロスサイトスクリプティング）攻撃に対して脆弱です。
 * localStorage に保存されたトークンは、ページ上で実行されるすべての JavaScript から
 * アクセス可能であり、悪意のあるスクリプトがトークンを窃取するリスクがあります。
 *
 * 本番環境では、以下のアプローチを検討してください:
 * - **BFF（Backend for Frontend）パターン**: サーバーサイドでトークンを管理し、
 *   クライアントには HTTP-only Cookie でセッションを発行する。
 *   トークンがブラウザの JavaScript から一切アクセスできなくなるため、
 *   XSS 攻撃によるトークン窃取を防止できます。
 * - **HTTP-only Cookie**: `HttpOnly`、`Secure`、`SameSite=Strict` 属性を設定した
 *   Cookie にトークンを保存することで、JavaScript からのアクセスを遮断します。
 *
 * 詳細は `docs/architecture/auth/token-storage-security.md` を参照してください。
 */
export class DevLocalStorageTokenStore implements TokenStore {
  private readonly tokenKey = 'k1s0_auth_tokens';
  private readonly verifierKey = 'k1s0_pkce_verifier';
  private readonly stateKey = 'k1s0_oauth_state';

  // コンストラクタで本番環境使用を検知し、エラーをスローする。
  // NODE_ENV が development/test 以外の場合はインスタンス化を拒否し、XSS によるトークン窃取を防ぐ。
  // POLY-004 監査対応: console.warn（継続動作）から throw Error（強制停止）に変更。
  constructor() {
    if (typeof window !== 'undefined' && !this._isDevEnvironment()) {
      throw new Error(
        '[k1s0-auth] DevLocalStorageTokenStore は開発・テスト専用です。' +
        '本番環境での使用は禁止されています。SecureTokenStore を使用してください。'
      );
    }
  }

  // NODE_ENV が development または test であるかを確認する。
  // NODE_ENV が未定義の場合は本番環境として扱う（安全側のデフォルト: 未定義時に開発扱いにしない）
  private _isDevEnvironment(): boolean {
    const env = process.env['NODE_ENV'] ?? 'production';
    return env === 'development' || env === 'test';
  }

  getTokenSet(): TokenSet | null {
    try {
      const data = localStorage.getItem(this.tokenKey);
      return data ? (JSON.parse(data) as TokenSet) : null;
    } catch {
      return null;
    }
  }

  // HIGH-FE-001 対応: setTokenSet を async Promise<void> に変更する（インターフェース準拠）
  async setTokenSet(tokenSet: TokenSet): Promise<void> {
    localStorage.setItem(this.tokenKey, JSON.stringify(tokenSet));
  }

  clearTokenSet(): void {
    localStorage.removeItem(this.tokenKey);
  }

  getCodeVerifier(): string | null {
    return sessionStorage.getItem(this.verifierKey);
  }

  setCodeVerifier(verifier: string): void {
    sessionStorage.setItem(this.verifierKey, verifier);
  }

  clearCodeVerifier(): void {
    sessionStorage.removeItem(this.verifierKey);
  }

  getState(): string | null {
    return sessionStorage.getItem(this.stateKey);
  }

  setState(state: string): void {
    sessionStorage.setItem(this.stateKey, state);
  }

  clearState(): void {
    sessionStorage.removeItem(this.stateKey);
  }
}

// M-010 監査対応: トークン保存失敗時に呼び出し元（React 等）に通知するコールバックオプション
export interface SecureTokenStoreOptions {
  /** トークン保存/削除に失敗した場合に呼び出されるコールバック */
  onTokenSetFailure?: (err: unknown) => void;
}

/**
 * H-007 監査対応: 本番環境用の安全なトークンストア
 * BFF（Backend for Frontend）パターンが必須: トークンは httpOnly Cookie に保存されるため
 * このクラスは直接トークンを保管せず、BFF サーバー経由のみアクセス可能。
 *
 * @requires BFF サーバーが以下のエンドポイントを実装していること:
 *   - GET  /bff/token    → TokenSet を返す（httpOnly Cookie セッションで認証）
 *   - POST /bff/token    → TokenSet を保存する
 *   - DELETE /bff/token  → TokenSet を削除する
 *   - GET  /bff/verifier → PKCE verifier を返す
 *   - POST /bff/verifier → PKCE verifier を保存する
 *   - DELETE /bff/verifier → PKCE verifier を削除する
 *   - GET  /bff/state    → OAuth state を返す
 *   - POST /bff/state    → OAuth state を保存する
 *   - DELETE /bff/state  → OAuth state を削除する
 *
 * @security XSS 対策
 * ブラウザ側 JavaScript はトークンへ一切アクセスできない。
 * BFF サーバーが httpOnly Cookie でセッションを管理し、トークンはサーバーサイドのみに保持する。
 */
export class SecureTokenStore implements TokenStore {
  // BFF エンドポイントのベース URL（デフォルトは同一オリジンの /bff）
  private readonly bffBaseUrl: string;
  // M-010 監査対応: トークン操作失敗時の通知コールバックを保持するオプション
  private readonly _options?: SecureTokenStoreOptions;
  // HIGH-FE-002 対応: インメモリキャッシュによる同期 getTokenSet() をサポートする。
  // BFF への非同期通信なしで認証状態を素早く確認できるようにする。
  // setTokenSet() でキャッシュを更新し、clearTokenSet() でクリアする。
  private _cachedTokenSet: TokenSet | null = null;

  constructor(bffBaseUrl = '/bff', options?: SecureTokenStoreOptions) {
    // BFF が利用できない環境（SSR 等）での誤使用を検知してエラーをスローする。
    // window が存在しない場合は BFF Cookie ベースの認証フローが成立しないため使用不可とする。
    if (typeof window === 'undefined') {
      throw new Error(
        '[k1s0-auth] SecureTokenStore はブラウザ環境でのみ使用可能です。' +
          'SSR 環境では MemoryTokenStore を使用してください。'
      );
    }
    this.bffBaseUrl = bffBaseUrl;
    this._options = options;
  }

  // M-011 監査対応: BFF エンドポイントへ fetch リクエストを送信する共通ヘルパー。
  // credentials: 'include' により httpOnly Cookie が自動送信される。
  // BFF が応答しない場合は意味のあるエラーメッセージをスローする。
  // validate 引数が指定された場合はレスポンスの実行時型検証を行う（未指定時は型アサーションのみ）。
  private async _request<T>(
    method: string,
    path: string,
    body?: unknown,
    validate?: (data: unknown) => T
  ): Promise<T | null> {
    const url = `${this.bffBaseUrl}${path}`;
    const init: RequestInit = {
      method,
      // httpOnly Cookie を自動的に送受信するために credentials: 'include' が必須
      credentials: 'include',
      headers: body !== undefined ? { 'Content-Type': 'application/json' } : {},
      body: body !== undefined ? JSON.stringify(body) : undefined,
    };
    let res: Response;
    try {
      res = await fetch(url, init);
    } catch (cause) {
      throw new Error(
        `[k1s0-auth] SecureTokenStore: BFF サーバー (${url}) へ接続できません。` +
          'BFF サーバーが起動しているか確認してください。',
        { cause }
      );
    }
    if (res.status === 404) return null;
    if (!res.ok) {
      throw new Error(
        `[k1s0-auth] SecureTokenStore: BFF サーバーがエラーを返しました (${res.status} ${res.statusText})。` +
          `エンドポイント: ${method} ${url}`
      );
    }
    if (method === 'GET' && res.status !== 204) {
      // M-011 監査対応: バリデータが指定されている場合は実行時型検証を行い、
      // 未指定の場合は型アサーションのみとする（後方互換性を維持）
      const data: unknown = await res.json();
      return validate ? validate(data) : (data as T);
    }
    return null;
  }

  // HIGH-FE-002 対応: インメモリキャッシュからトークンセットを返す。
  // setTokenSet() によりキャッシュが更新されている場合はキャッシュ値を返す。
  // キャッシュが null の場合（初回起動時等）は null を返す。
  // 完全な非同期取得が必要な場合は getTokenSetAsync() を使用すること。
  getTokenSet(): TokenSet | null {
    return this._cachedTokenSet;
  }

  // BFF 経由でトークンセットを非同期取得する（推奨: 本番用途はこちらを使用）。
  // HIGH-FE-002 対応: BFF から取得したトークンをインメモリキャッシュに反映する。
  async getTokenSetAsync(): Promise<TokenSet | null> {
    const tokenSet = await this._request<TokenSet>('GET', '/token');
    // BFF からの最新値でキャッシュを更新する（null の場合もクリアする）
    this._cachedTokenSet = tokenSet;
    return tokenSet;
  }

  // BFF 経由でトークンセットを保存する（httpOnly Cookie セッションに紐付けて BFF が保管する）。
  // HIGH-FE-001 対応: setTokenSet を async Promise<void> に変更する。
  // BFF への非同期通信を await できるようにし、保存失敗を呼び出し元に伝播する。
  // HIGH-FE-002 対応: setTokenSet 実行時にインメモリキャッシュも更新する。
  async setTokenSet(tokenSet: TokenSet): Promise<void> {
    // キャッシュを先に更新して同期アクセスで即座に反映されるようにする
    this._cachedTokenSet = tokenSet;
    try {
      await this._request('POST', '/token', tokenSet);
    } catch (err: unknown) {
      console.error('[k1s0-auth] SecureTokenStore: トークンの保存に失敗しました。', err);
      // M-010 監査対応: 呼び出し元に認証リセットを通知する
      this._options?.onTokenSetFailure?.(err);
      throw err;
    }
  }

  // BFF 経由でトークンセットを削除する。
  // HIGH-FE-002 対応: clearTokenSet 実行時にインメモリキャッシュもクリアする。
  clearTokenSet(): void {
    // キャッシュを即座にクリアして、BFF 通信完了を待たずに認証状態を反映する
    this._cachedTokenSet = null;
    this._request('DELETE', '/token').catch((err: unknown) => {
      console.error('[k1s0-auth] SecureTokenStore: トークンの削除に失敗しました。', err);
      // M-010 監査対応: 呼び出し元に認証リセットを通知する
      this._options?.onTokenSetFailure?.(err);
    });
  }

  // BFF 経由で PKCE code verifier を取得する（同期インターフェース制約のため null を返す）。
  getCodeVerifier(): string | null {
    // PKCE フローでは getCodeVerifierAsync() を使用すること。
    return null;
  }

  // BFF 経由で PKCE code verifier を非同期取得する。
  async getCodeVerifierAsync(): Promise<string | null> {
    const result = await this._request<{ verifier: string }>('GET', '/verifier');
    return result?.verifier ?? null;
  }

  // BFF 経由で PKCE code verifier を保存する。
  setCodeVerifier(verifier: string): void {
    this._request('POST', '/verifier', { verifier }).catch((err: unknown) => {
      console.error('[k1s0-auth] SecureTokenStore: PKCE verifier の保存に失敗しました。', err);
    });
  }

  // BFF 経由で PKCE code verifier を削除する。
  clearCodeVerifier(): void {
    this._request('DELETE', '/verifier').catch((err: unknown) => {
      console.error('[k1s0-auth] SecureTokenStore: PKCE verifier の削除に失敗しました。', err);
    });
  }

  // BFF 経由で OAuth state を取得する（同期インターフェース制約のため null を返す）。
  getState(): string | null {
    // OAuth state フローでは getStateAsync() を使用すること。
    return null;
  }

  // BFF 経由で OAuth state を非同期取得する。
  async getStateAsync(): Promise<string | null> {
    const result = await this._request<{ state: string }>('GET', '/state');
    return result?.state ?? null;
  }

  // BFF 経由で OAuth state を保存する。
  setState(state: string): void {
    this._request('POST', '/state', { state }).catch((err: unknown) => {
      console.error('[k1s0-auth] SecureTokenStore: OAuth state の保存に失敗しました。', err);
    });
  }

  // BFF 経由で OAuth state を削除する。
  clearState(): void {
    this._request('DELETE', '/state').catch((err: unknown) => {
      console.error('[k1s0-auth] SecureTokenStore: OAuth state の削除に失敗しました。', err);
    });
  }
}
