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
 * localStorage ベースのトークンストア。
 * ブラウザ環境で使用する。
 * PKCE verifier と state は sessionStorage に保存する（タブ間の分離のため）。
 */
export class LocalStorageTokenStore implements TokenStore {
  private readonly tokenKey = 'k1s0_auth_tokens';
  private readonly verifierKey = 'k1s0_pkce_verifier';
  private readonly stateKey = 'k1s0_oauth_state';

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
