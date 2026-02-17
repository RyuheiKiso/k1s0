/**
 * トークン保存ストア
 * メモリストアと localStorage ストアの 2 種類を提供する。
 */
import type { TokenSet, TokenStore } from './types.js';
/**
 * メモリベースのトークンストア。
 * テスト用、または SSR 環境で使用する。
 */
export declare class MemoryTokenStore implements TokenStore {
    private tokenSet;
    private codeVerifier;
    private state;
    getTokenSet(): TokenSet | null;
    setTokenSet(tokenSet: TokenSet): void;
    clearTokenSet(): void;
    getCodeVerifier(): string | null;
    setCodeVerifier(verifier: string): void;
    clearCodeVerifier(): void;
    getState(): string | null;
    setState(state: string): void;
    clearState(): void;
}
/**
 * localStorage ベースのトークンストア。
 * ブラウザ環境で使用する。
 * PKCE verifier と state は sessionStorage に保存する（タブ間の分離のため）。
 */
export declare class LocalStorageTokenStore implements TokenStore {
    private readonly tokenKey;
    private readonly verifierKey;
    private readonly stateKey;
    getTokenSet(): TokenSet | null;
    setTokenSet(tokenSet: TokenSet): void;
    clearTokenSet(): void;
    getCodeVerifier(): string | null;
    setCodeVerifier(verifier: string): void;
    clearCodeVerifier(): void;
    getState(): string | null;
    setState(state: string): void;
    clearState(): void;
}
//# sourceMappingURL=token-store.d.ts.map