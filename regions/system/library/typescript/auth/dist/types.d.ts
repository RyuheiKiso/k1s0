/**
 * OAuth2 PKCE クライアント用の型定義
 * 認証認可設計.md の JWT Claims 構造に準拠
 */
/** AuthClient の設定 */
export interface AuthConfig {
    /** OIDC Discovery URL */
    discoveryUrl: string;
    /** OAuth2 クライアント ID */
    clientId: string;
    /** リダイレクト URI */
    redirectUri: string;
    /** 要求するスコープ */
    scopes: string[];
    /** Keycloak の logout endpoint URL（オプション） */
    logoutUrl?: string;
    /** post_logout_redirect_uri（オプション） */
    postLogoutRedirectUri?: string;
}
/** トークンエンドポイントのレスポンス */
export interface TokenResponse {
    access_token: string;
    refresh_token: string;
    id_token: string;
    expires_in: number;
    token_type: string;
    scope?: string;
}
/** 保存用のトークンセット */
export interface TokenSet {
    accessToken: string;
    refreshToken: string;
    idToken: string;
    expiresAt: number;
}
/** JWT Claims 構造（認証認可設計.md 準拠） */
export interface Claims {
    sub: string;
    iss: string;
    aud: string;
    exp: number;
    iat: number;
    jti: string;
    typ: string;
    azp: string;
    scope: string;
    preferred_username: string;
    email: string;
    realm_access: {
        roles: string[];
    };
    resource_access: Record<string, {
        roles: string[];
    }>;
    tier_access: string[];
}
/** OIDC Discovery レスポンス */
export interface OIDCDiscovery {
    authorization_endpoint: string;
    token_endpoint: string;
    end_session_endpoint: string;
    jwks_uri: string;
    issuer: string;
}
/** 認証状態変更コールバック */
export type AuthStateCallback = (authenticated: boolean) => void;
/** トークンストアのインターフェース */
export interface TokenStore {
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
//# sourceMappingURL=types.d.ts.map