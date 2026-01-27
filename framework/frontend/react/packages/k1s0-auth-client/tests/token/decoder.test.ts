/**
 * トークンデコーダーのテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  decodeToken,
  isTokenValid,
  needsRefresh,
  getTimeUntilExpiry,
  claimsToUser,
} from '../../src/token/decoder';
import type { Claims } from '../../src/types';

// テスト用の JWT トークンを生成するヘルパー
// 実際の署名は不要（jose の decodeJwt は署名検証しない）
const createTestToken = (payload: Record<string, unknown>): string => {
  const header = { alg: 'HS256', typ: 'JWT' };
  const headerBase64 = Buffer.from(JSON.stringify(header)).toString('base64url');
  const payloadBase64 = Buffer.from(JSON.stringify(payload)).toString('base64url');
  const signature = 'test-signature';
  return `${headerBase64}.${payloadBase64}.${signature}`;
};

describe('decodeToken', () => {
  it('有効なトークンをデコードできること', () => {
    const payload = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 3600,
      iat: Math.floor(Date.now() / 1000),
      roles: ['user'],
      permissions: ['read'],
      tenant_id: 'tenant-1',
    };
    const token = createTestToken(payload);

    const result = decodeToken(token);

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.claims.sub).toBe('user-123');
      expect(result.claims.roles).toEqual(['user']);
      expect(result.claims.permissions).toEqual(['read']);
    }
  });

  it('無効なトークン形式でエラーを返すこと', () => {
    const result = decodeToken('invalid-token');

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe('INVALID_TOKEN');
    }
  });

  it('必須フィールドがない場合エラーを返すこと', () => {
    const payload = {
      // sub がない
      exp: Math.floor(Date.now() / 1000) + 3600,
    };
    const token = createTestToken(payload);

    const result = decodeToken(token);

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe('INVALID_TOKEN');
    }
  });
});

describe('isTokenValid', () => {
  it('有効期限内のトークンは true を返すこと', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 3600, // 1時間後
      iat: Math.floor(Date.now() / 1000),
      roles: [],
      permissions: [],
    };

    expect(isTokenValid(claims, 0)).toBe(true);
  });

  it('有効期限切れのトークンは false を返すこと', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) - 100, // 過去
      iat: Math.floor(Date.now() / 1000) - 3700,
      roles: [],
      permissions: [],
    };

    expect(isTokenValid(claims, 0)).toBe(false);
  });

  it('マージン時間を考慮して判定すること', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 30, // 30秒後
      iat: Math.floor(Date.now() / 1000),
      roles: [],
      permissions: [],
    };

    // 60秒のマージンを指定すると無効と判定
    expect(isTokenValid(claims, 60000)).toBe(false);

    // 10秒のマージンなら有効
    expect(isTokenValid(claims, 10000)).toBe(true);
  });
});

describe('needsRefresh', () => {
  it('マージン時間内に入った場合 true を返すこと', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 30, // 30秒後
      iat: Math.floor(Date.now() / 1000),
      roles: [],
      permissions: [],
    };

    // 60秒のマージンなので、30秒後に期限切れならリフレッシュ必要
    expect(needsRefresh(claims, 60000)).toBe(true);
  });

  it('マージン時間外の場合 false を返すこと', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 3600, // 1時間後
      iat: Math.floor(Date.now() / 1000),
      roles: [],
      permissions: [],
    };

    // 60秒のマージンに入っていない
    expect(needsRefresh(claims, 60000)).toBe(false);
  });
});

describe('getTimeUntilExpiry', () => {
  it('残り時間をミリ秒で返すこと', () => {
    const expiresInSeconds = 3600;
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + expiresInSeconds,
      iat: Math.floor(Date.now() / 1000),
      roles: [],
      permissions: [],
    };

    const timeUntil = getTimeUntilExpiry(claims);

    // 誤差を許容（テスト実行時間分）
    expect(timeUntil).toBeGreaterThan((expiresInSeconds - 1) * 1000);
    expect(timeUntil).toBeLessThanOrEqual(expiresInSeconds * 1000);
  });

  it('期限切れの場合は負の値を返すこと', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) - 100, // 100秒前に期限切れ
      iat: Math.floor(Date.now() / 1000) - 3700,
      roles: [],
      permissions: [],
    };

    const timeUntil = getTimeUntilExpiry(claims);

    expect(timeUntil).toBeLessThan(0);
  });
});

describe('claimsToUser', () => {
  it('Claims から AuthUser を構築すること', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 3600,
      iat: Math.floor(Date.now() / 1000),
      roles: ['admin', 'user'],
      permissions: ['read', 'write'],
      tenant_id: 'tenant-1',
    };

    const user = claimsToUser(claims);

    expect(user.id).toBe('user-123');
    expect(user.roles).toEqual(['admin', 'user']);
    expect(user.permissions).toEqual(['read', 'write']);
    expect(user.tenantId).toBe('tenant-1');
    expect(user.claims).toBe(claims);
  });

  it('tenant_id がない場合も正しく構築すること', () => {
    const claims: Claims = {
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp: Math.floor(Date.now() / 1000) + 3600,
      iat: Math.floor(Date.now() / 1000),
      roles: ['user'],
      permissions: ['read'],
    };

    const user = claimsToUser(claims);

    expect(user.id).toBe('user-123');
    expect(user.tenantId).toBeUndefined();
  });
});
