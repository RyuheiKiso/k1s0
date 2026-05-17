// k1s0 tier3 BFF cookie auth
// access_token を tier3 に公開しない設計（forbidden_export_symbols 整合 3 の物理根拠）
// Keycloak OIDC + BFF auth-edge パターン
// 注意: access_token / refresh_token の getter を一切公開しない

// 認証状態の型（tier3 が保持する最小限の情報）
export interface AuthState {
  // 認証済みか否か（true = BFF cookie が有効）
  readonly authenticated: boolean;
  // テナント ID は OIDC token claim tid からのみ取得する（直接引数禁止）
  // 型で tenant_id を API に露出しないことを保証する
  readonly tenantId?: never;
  // actor ID（表示用のみ。認可ロジックには使用しない）
  readonly actorDisplayName: string | null;
  // BFF セッション有効期限（表示用のみ）
  readonly sessionExpiresAt: string | null;
}

// 認証されていない状態の初期値
export const UNAUTHENTICATED: AuthState = {
  authenticated: false,
  actorDisplayName: null,
  sessionExpiresAt: null,
};

// BFF auth-edge の cookie 状態を確認するエンドポイント（BFF が検証する）
// access_token を tier3 に返さない設計
export type AuthCheckResult =
  // 認証済み
  | { readonly status: "authenticated"; readonly displayName: string; readonly expiresAt: string }
  // 未認証
  | { readonly status: "unauthenticated" }
  // step_up 必要（WebAuthn / TOTP）
  | { readonly status: "step_up_required"; readonly stepUpMethod: StepUpMethod };

// step_up の方法（3 種）
export type StepUpMethod = "webauthn" | "totp" | "fido2";

// テナント切り替え（4 layer storage 全 purge を実施する設計）
// tenant_id を直接受け取らない（BFF が claim から取得する）
export interface TenantSwitchRequest {
  // 切り替え先 tenant の slug（display 用のみ）
  readonly targetTenantSlug: string;
}

// OIDC Back-Channel Logout 受信後に broadcast するイベント型
export interface LogoutBroadcastEvent {
  // ブロードキャスト種別
  readonly type: "logout_broadcast";
  // ログアウト対象 session の hash（個人情報を含まない）
  readonly sessionHash: string;
}
