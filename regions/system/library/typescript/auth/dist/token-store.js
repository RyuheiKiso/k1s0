/**
 * トークン保存ストア
 * メモリストアと localStorage ストアの 2 種類を提供する。
 */
/**
 * メモリベースのトークンストア。
 * テスト用、または SSR 環境で使用する。
 */
export class MemoryTokenStore {
    tokenSet = null;
    codeVerifier = null;
    state = null;
    getTokenSet() {
        return this.tokenSet;
    }
    setTokenSet(tokenSet) {
        this.tokenSet = tokenSet;
    }
    clearTokenSet() {
        this.tokenSet = null;
    }
    getCodeVerifier() {
        return this.codeVerifier;
    }
    setCodeVerifier(verifier) {
        this.codeVerifier = verifier;
    }
    clearCodeVerifier() {
        this.codeVerifier = null;
    }
    getState() {
        return this.state;
    }
    setState(state) {
        this.state = state;
    }
    clearState() {
        this.state = null;
    }
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
export class LocalStorageTokenStore {
    tokenKey = 'k1s0_auth_tokens';
    verifierKey = 'k1s0_pkce_verifier';
    stateKey = 'k1s0_oauth_state';
    getTokenSet() {
        try {
            const data = localStorage.getItem(this.tokenKey);
            return data ? JSON.parse(data) : null;
        }
        catch {
            return null;
        }
    }
    setTokenSet(tokenSet) {
        localStorage.setItem(this.tokenKey, JSON.stringify(tokenSet));
    }
    clearTokenSet() {
        localStorage.removeItem(this.tokenKey);
    }
    getCodeVerifier() {
        return sessionStorage.getItem(this.verifierKey);
    }
    setCodeVerifier(verifier) {
        sessionStorage.setItem(this.verifierKey, verifier);
    }
    clearCodeVerifier() {
        sessionStorage.removeItem(this.verifierKey);
    }
    getState() {
        return sessionStorage.getItem(this.stateKey);
    }
    setState(state) {
        sessionStorage.setItem(this.stateKey, state);
    }
    clearState() {
        sessionStorage.removeItem(this.stateKey);
    }
}
//# sourceMappingURL=token-store.js.map