/**
 * OAuth2 Authorization Code + PKCE クライアント
 * Keycloak 対応のクライアント側認証ライブラリ
 */
import type { AuthConfig, AuthStateCallback, TokenSet, TokenStore } from './types.js';
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
export declare class AuthClient {
    private readonly config;
    private readonly tokenStore;
    private readonly fetchFn;
    private readonly redirectFn;
    private readonly generateStateFn;
    private listeners;
    private refreshTimer;
    private discoveryCache;
    constructor(options: AuthClientOptions);
    /**
     * OAuth2 Authorization Code + PKCE フローを開始する。
     * 1. code_verifier を生成
     * 2. code_challenge を計算
     * 3. authorize URL を構築
     * 4. リダイレクト
     */
    login(): Promise<void>;
    /**
     * 認可コールバックを処理する。
     * code + code_verifier で token endpoint に POST してトークンを取得・保存する。
     * @param code - 認可コード
     * @param state - state パラメータ（CSRF 対策検証用）
     * @returns トークンセット
     */
    handleCallback(code: string, state: string): Promise<TokenSet>;
    /**
     * refresh_token で新しいアクセストークンを取得する。
     */
    refreshToken(): Promise<void>;
    /**
     * 有効なアクセストークンを返す。
     * 期限切れ 60 秒前なら自動リフレッシュする。
     */
    getAccessToken(): Promise<string>;
    /**
     * 認証状態を返す。
     */
    isAuthenticated(): boolean;
    /**
     * ログアウト処理。
     * トークンを削除し、Keycloak の logout endpoint にリダイレクトする。
     */
    logout(): Promise<void>;
    /**
     * 現在のトークンセットを取得する。
     */
    getTokenSet(): TokenSet | null;
    /**
     * 認証状態の変更を監視するリスナーを登録する。
     * @returns リスナーの登録解除関数
     */
    onAuthStateChange(callback: AuthStateCallback): () => void;
    /**
     * リフレッシュをスケジュールする。
     * トークンの有効期限 60 秒前にリフレッシュを実行する。
     */
    private scheduleRefresh;
    private notifyListeners;
    private fetchDiscovery;
}
export declare class AuthError extends Error {
    constructor(message: string);
}
//# sourceMappingURL=auth-client.d.ts.map