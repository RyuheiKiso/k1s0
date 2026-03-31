/**
 * トークン保存ストア
 * メモリストアと localStorage ストアの 2 種類を提供する。
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

  setTokenSet(tokenSet: TokenSet): void {
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
  private _isDevEnvironment(): boolean {
    return process.env['NODE_ENV'] === 'development' || process.env['NODE_ENV'] === 'test';
  }

  getTokenSet(): TokenSet | null {
    try {
      const data = localStorage.getItem(this.tokenKey);
      return data ? (JSON.parse(data) as TokenSet) : null;
    } catch {
      return null;
    }
  }

  setTokenSet(tokenSet: TokenSet): void {
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
