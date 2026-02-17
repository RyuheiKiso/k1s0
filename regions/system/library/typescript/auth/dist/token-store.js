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