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
 *
 * @security セキュリティ警告
 *
 * **この実装は開発・テスト用途のみを想定しています。本番環境での使用は推奨しません。**
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