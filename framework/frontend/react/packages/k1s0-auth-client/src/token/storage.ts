import type { TokenPair, TokenStorage } from '../types.js';

const STORAGE_KEY = 'k1s0_auth_tokens';

/**
 * SessionStorage を使用するトークンストレージ
 * ブラウザタブを閉じるとトークンが消える
 */
export class SessionTokenStorage implements TokenStorage {
  private readonly key: string;

  constructor(key: string = STORAGE_KEY) {
    this.key = key;
  }

  get(): TokenPair | null {
    if (typeof window === 'undefined') return null;
    try {
      const stored = sessionStorage.getItem(this.key);
      if (!stored) return null;
      return JSON.parse(stored) as TokenPair;
    } catch {
      return null;
    }
  }

  set(tokens: TokenPair): void {
    if (typeof window === 'undefined') return;
    try {
      sessionStorage.setItem(this.key, JSON.stringify(tokens));
    } catch {
      // ストレージ書き込み失敗時は無視
    }
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    try {
      sessionStorage.removeItem(this.key);
    } catch {
      // ストレージ削除失敗時は無視
    }
  }
}

/**
 * LocalStorage を使用するトークンストレージ
 * ブラウザを閉じてもトークンが永続化される（"Remember me" 用）
 */
export class LocalTokenStorage implements TokenStorage {
  private readonly key: string;

  constructor(key: string = STORAGE_KEY) {
    this.key = key;
  }

  get(): TokenPair | null {
    if (typeof window === 'undefined') return null;
    try {
      const stored = localStorage.getItem(this.key);
      if (!stored) return null;
      return JSON.parse(stored) as TokenPair;
    } catch {
      return null;
    }
  }

  set(tokens: TokenPair): void {
    if (typeof window === 'undefined') return;
    try {
      localStorage.setItem(this.key, JSON.stringify(tokens));
    } catch {
      // ストレージ書き込み失敗時は無視
    }
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    try {
      localStorage.removeItem(this.key);
    } catch {
      // ストレージ削除失敗時は無視
    }
  }
}

/**
 * インメモリトークンストレージ
 * テスト用またはセキュリティ要件が高い場合に使用
 */
export class MemoryTokenStorage implements TokenStorage {
  private tokens: TokenPair | null = null;

  get(): TokenPair | null {
    return this.tokens;
  }

  set(tokens: TokenPair): void {
    this.tokens = tokens;
  }

  clear(): void {
    this.tokens = null;
  }
}
