import { decodeJwt } from 'jose';
import { ClaimsSchema, type Claims, type AuthError } from '../types.js';

/**
 * JWT デコード結果
 */
export type DecodeResult =
  | { success: true; claims: Claims }
  | { success: false; error: AuthError };

/**
 * JWT トークンをデコードして Claims を取得
 * 署名の検証は行わない（サーバー側で検証済みの前提）
 *
 * @param token JWT トークン
 * @returns デコード結果
 */
export function decodeToken(token: string): DecodeResult {
  try {
    const payload = decodeJwt(token);

    // Claims スキーマでバリデーション
    const result = ClaimsSchema.safeParse(payload);

    if (!result.success) {
      return {
        success: false,
        error: {
          code: 'INVALID_TOKEN',
          message: `Invalid token claims: ${result.error.message}`,
        },
      };
    }

    return { success: true, claims: result.data };
  } catch (err) {
    return {
      success: false,
      error: {
        code: 'INVALID_TOKEN',
        message: err instanceof Error ? err.message : 'Failed to decode token',
        cause: err instanceof Error ? err : undefined,
      },
    };
  }
}

/**
 * トークンの有効期限を確認
 *
 * @param claims JWT Claims
 * @param marginMs 有効期限前のマージン（ms）
 * @returns 有効期限内なら true
 */
export function isTokenValid(claims: Claims, marginMs: number = 0): boolean {
  const now = Math.floor(Date.now() / 1000);
  const exp = claims.exp;
  const marginSec = Math.floor(marginMs / 1000);

  return now < exp - marginSec;
}

/**
 * トークンがリフレッシュ必要かどうかを確認
 *
 * @param claims JWT Claims
 * @param marginMs 有効期限前のマージン（ms）
 * @returns リフレッシュが必要なら true
 */
export function needsRefresh(claims: Claims, marginMs: number): boolean {
  const now = Math.floor(Date.now() / 1000);
  const exp = claims.exp;
  const marginSec = Math.floor(marginMs / 1000);

  // 有効期限前のマージン時間に入ったらリフレッシュ
  return now >= exp - marginSec;
}

/**
 * トークンの残り有効時間を取得（ms）
 *
 * @param claims JWT Claims
 * @returns 残り有効時間（ms）。期限切れの場合は負の値
 */
export function getTimeUntilExpiry(claims: Claims): number {
  const now = Math.floor(Date.now() / 1000);
  return (claims.exp - now) * 1000;
}

/**
 * Claims から AuthUser を構築
 */
export function claimsToUser(claims: Claims) {
  return {
    id: claims.sub,
    roles: claims.roles,
    permissions: claims.permissions,
    tenantId: claims.tenant_id,
    claims,
  };
}
