// k1s0 tier3 BFF cookie scope 検証ユーティリティ
// Domain=tier3.<tenant>.<root> の cookie scope が正しいことを確認する
// tier3/CLAUDE.md: public type に tenant_id フィールド禁止（BFF が claim から取得する）
// cookie scope 検証はセキュリティ上必須（テナント間 cookie 漏洩を防ぐ）

// k1s0 tier3 BFF cookie のドメインプレフィックス
const TIER3_SUBDOMAIN_PREFIX = "tier3.";
// cookie scope 検証に使用するドットセパレータ
const DOMAIN_SEPARATOR = ".";

// cookie scope 検証結果の型
export type CookieScopeResult =
  // スコープが正しい
  | { readonly valid: true }
  // スコープが不正（理由を含む）
  | { readonly valid: false; readonly reason: string };

// tier3 BFF cookie のドメイン形式を検証する
// expected format: tier3.<tenantSlug>.<rootDomain>
// 例: tier3.acme.k1s0.example.com
export function validateCookieDomain(
  cookieDomain: string,
  currentHostname: string,
): CookieScopeResult {
  // cookieDomain が空の場合は無効とみなす
  if (!cookieDomain) {
    return { valid: false, reason: "cookie domain is empty" };
  }
  // cookie domain が tier3. で始まることを確認する
  if (!cookieDomain.startsWith(TIER3_SUBDOMAIN_PREFIX)) {
    // tier3 サブドメインプレフィックスがない場合はスコープが不正
    return {
      valid: false,
      reason: `cookie domain '${cookieDomain}' does not start with '${TIER3_SUBDOMAIN_PREFIX}'`,
    };
  }
  // 現在のホスト名が cookie domain のサブドメインであることを確認する
  // (例: currentHostname=app.tier3.acme.k1s0.example.com は tier3.acme.k1s0.example.com のスコープに含まれる)
  const isSubdomain =
    currentHostname === cookieDomain ||
    currentHostname.endsWith(`${DOMAIN_SEPARATOR}${cookieDomain}`);
  // サブドメイン関係にない場合はスコープが不正
  if (!isSubdomain) {
    return {
      valid: false,
      reason: `hostname '${currentHostname}' is not within cookie domain scope '${cookieDomain}'`,
    };
  }
  // cookie domain が有効なセグメント数（最低 3 セグメント: tier3 + tenant + root）を持つことを確認する
  const segments = cookieDomain.split(DOMAIN_SEPARATOR);
  // セグメント数が 3 未満の場合はスコープが不正（tier3.<tenant>.<root> 最低形式）
  if (segments.length < 3) {
    return {
      valid: false,
      reason: `cookie domain '${cookieDomain}' has insufficient segments (minimum: tier3.<tenant>.<root>)`,
    };
  }
  // 全セグメントが空でないことを確認する
  for (const segment of segments) {
    // 空セグメントはドメイン不正
    if (!segment) {
      return {
        valid: false,
        reason: `cookie domain '${cookieDomain}' contains empty segment`,
      };
    }
  }
  // スコープが正しい
  return { valid: true };
}

// 現在のブラウザの cookie が tier3 BFF の正しいスコープを持つかどうかを確認する
// document.cookie から Set-Cookie ヘッダーを直接読むことはできないため、
// BFF auth check レスポンスに含まれる cookie domain ヒントを利用する
export function validateBffCookieScope(
  bffCookieDomainHint: string,
): CookieScopeResult {
  // ブラウザ環境でない場合は検証をスキップする（SSR / テスト環境対策）
  if (typeof window === "undefined" || typeof window.location === "undefined") {
    // window が存在しない場合はスキップ（valid とみなす）
    return { valid: true };
  }
  // 現在のホスト名を取得する（location.hostname はポート番号を含まない）
  const currentHostname = window.location.hostname;
  // cookie domain を検証する
  return validateCookieDomain(bffCookieDomainHint, currentHostname);
}

// tier3 BFF cookie が HttpOnly か確認するヒューリスティック（完全な検証は不可能）
// document.cookie で読めた場合は HttpOnly でない（セキュリティ問題）
export function warnIfCookieAccessible(cookieName: string): boolean {
  // ブラウザ環境でない場合は false を返す
  if (typeof document === "undefined") {
    return false;
  }
  // document.cookie から cookie を検索する
  const cookies = document.cookie.split(";");
  // cookie 名にマッチするエントリを探す
  for (const cookie of cookies) {
    // cookie 名と値を分割する
    const [name] = cookie.trim().split("=");
    // cookie 名が一致する場合は HttpOnly でない（警告）
    if (name?.trim() === cookieName) {
      // JS から読めた場合は HttpOnly フラグがない（セキュリティ警告）
      return true;
    }
  }
  // JS から読めなかった場合は HttpOnly フラグが正しく設定されている
  return false;
}

// BFF session cookie の名前（tier3 SPA が認識するセッション cookie 名）
export const BFF_SESSION_COOKIE_NAME = "k1s0_bff_session";
