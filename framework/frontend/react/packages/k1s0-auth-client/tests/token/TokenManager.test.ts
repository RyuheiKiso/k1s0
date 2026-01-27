/**
 * TokenManager のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { TokenManager } from '../../src/token/TokenManager';
import type { TokenPair, TokenStorage, TokenRefresher } from '../../src/types';

// テスト用の JWT トークンを生成するヘルパー
const createTestToken = (payload: Record<string, unknown>): string => {
  const header = { alg: 'HS256', typ: 'JWT' };
  const headerBase64 = Buffer.from(JSON.stringify(header)).toString('base64url');
  const payloadBase64 = Buffer.from(JSON.stringify(payload)).toString('base64url');
  const signature = 'test-signature';
  return `${headerBase64}.${payloadBase64}.${signature}`;
};

// テスト用のトークンペアを作成
const createTestTokenPair = (expiresInSeconds: number = 3600): TokenPair => {
  const exp = Math.floor(Date.now() / 1000) + expiresInSeconds;
  return {
    accessToken: createTestToken({
      sub: 'user-123',
      iss: 'https://auth.example.com',
      exp,
      iat: Math.floor(Date.now() / 1000),
      roles: ['user'],
      permissions: ['read'],
    }),
    refreshToken: 'refresh-token',
  };
};

// モックストレージ
const createMockStorage = (): TokenStorage & { _tokens: TokenPair | null } => {
  let tokens: TokenPair | null = null;
  return {
    _tokens: null,
    get: vi.fn(() => tokens),
    set: vi.fn((t: TokenPair) => {
      tokens = t;
    }),
    clear: vi.fn(() => {
      tokens = null;
    }),
  };
};

describe('TokenManager', () => {
  let tokenManager: TokenManager;
  let mockStorage: ReturnType<typeof createMockStorage>;

  beforeEach(() => {
    vi.useFakeTimers();
    mockStorage = createMockStorage();
    tokenManager = new TokenManager({
      storage: mockStorage,
      autoRefresh: false, // テスト時は自動リフレッシュを無効化
    });
  });

  afterEach(() => {
    tokenManager.dispose();
    vi.useRealTimers();
  });

  describe('setTokens', () => {
    it('トークンを保存できること', () => {
      const tokens = createTestTokenPair();

      tokenManager.setTokens(tokens);

      expect(mockStorage.set).toHaveBeenCalledWith(tokens);
    });
  });

  describe('clearTokens', () => {
    it('トークンをクリアできること', () => {
      tokenManager.clearTokens();

      expect(mockStorage.clear).toHaveBeenCalled();
    });
  });

  describe('getTokens', () => {
    it('保存されたトークンを取得できること', () => {
      const tokens = createTestTokenPair();
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      const result = tokenManager.getTokens();

      expect(result).toEqual(tokens);
    });
  });

  describe('isValid', () => {
    it('有効なトークンがある場合 true を返すこと', () => {
      const tokens = createTestTokenPair(3600); // 1時間後に期限切れ
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      expect(tokenManager.isValid()).toBe(true);
    });

    it('トークンがない場合 false を返すこと', () => {
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(null);

      expect(tokenManager.isValid()).toBe(false);
    });

    it('期限切れのトークンの場合 false を返すこと', () => {
      const tokens = createTestTokenPair(-100); // 既に期限切れ
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      expect(tokenManager.isValid()).toBe(false);
    });
  });

  describe('canRefresh', () => {
    it('リフレッシュトークンとリフレッシャーがある場合 true を返すこと', () => {
      const refresher: TokenRefresher = vi.fn();
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        autoRefresh: false,
      });

      const tokens = createTestTokenPair();
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      expect(managerWithRefresh.canRefresh()).toBe(true);

      managerWithRefresh.dispose();
    });

    it('リフレッシュトークンがない場合 false を返すこと', () => {
      const refresher: TokenRefresher = vi.fn();
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        autoRefresh: false,
      });

      const tokens: TokenPair = {
        accessToken: createTestToken({
          sub: 'user-123',
          iss: 'https://auth.example.com',
          exp: Math.floor(Date.now() / 1000) + 3600,
          iat: Math.floor(Date.now() / 1000),
          roles: [],
          permissions: [],
        }),
        // refreshToken なし
      };
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      expect(managerWithRefresh.canRefresh()).toBe(false);

      managerWithRefresh.dispose();
    });
  });

  describe('getValidToken', () => {
    it('トークンがない場合 none を返すこと', async () => {
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(null);

      const result = await tokenManager.getValidToken();

      expect(result.type).toBe('none');
    });

    it('有効なトークンがある場合 valid を返すこと', async () => {
      const tokens = createTestTokenPair(3600);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      const result = await tokenManager.getValidToken();

      expect(result.type).toBe('valid');
      if (result.type === 'valid') {
        expect(result.token).toBe(tokens.accessToken);
        expect(result.claims).toBeDefined();
      }
    });

    it('リフレッシュが成功した場合 refreshed を返すこと', async () => {
      // 既存トークン（期限間近）
      const oldTokens = createTestTokenPair(30); // 30秒後に期限切れ
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      // 新しいトークン
      const newTokens = createTestTokenPair(3600);
      const refresher = vi.fn().mockResolvedValue(newTokens);

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000, // 60秒のマージン
        autoRefresh: false,
      });

      const result = await managerWithRefresh.getValidToken();

      expect(result.type).toBe('refreshed');
      if (result.type === 'refreshed') {
        expect(result.token).toBe(newTokens.accessToken);
      }
      expect(refresher).toHaveBeenCalled();

      managerWithRefresh.dispose();
    });

    it('リフレッシュが失敗した場合 expired を返すこと', async () => {
      const oldTokens = createTestTokenPair(30);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const refresher = vi.fn().mockResolvedValue(null);

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000,
        autoRefresh: false,
      });

      const result = await managerWithRefresh.getValidToken();

      expect(result.type).toBe('expired');

      managerWithRefresh.dispose();
    });
  });

  describe('forceRefresh', () => {
    it('強制的にトークンをリフレッシュできること', async () => {
      const oldTokens = createTestTokenPair(3600);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const newTokens = createTestTokenPair(7200);
      const refresher = vi.fn().mockResolvedValue(newTokens);

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        autoRefresh: false,
      });

      const result = await managerWithRefresh.forceRefresh();

      expect(result).toEqual(newTokens);
      expect(mockStorage.set).toHaveBeenCalledWith(newTokens);

      managerWithRefresh.dispose();
    });

    it('リフレッシュトークンがない場合 null を返すこと', async () => {
      const tokens: TokenPair = {
        accessToken: createTestToken({
          sub: 'user-123',
          iss: 'https://auth.example.com',
          exp: Math.floor(Date.now() / 1000) + 3600,
          iat: Math.floor(Date.now() / 1000),
          roles: [],
          permissions: [],
        }),
      };
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      const refresher = vi.fn();
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        autoRefresh: false,
      });

      const result = await managerWithRefresh.forceRefresh();

      expect(result).toBeNull();
      expect(refresher).not.toHaveBeenCalled();

      managerWithRefresh.dispose();
    });
  });

  describe('onRefresh', () => {
    it('リフレッシュイベントをリッスンできること', async () => {
      const oldTokens = createTestTokenPair(30);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const newTokens = createTestTokenPair(3600);
      const refresher = vi.fn().mockResolvedValue(newTokens);

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000,
        autoRefresh: false,
      });

      const listener = vi.fn();
      managerWithRefresh.onRefresh(listener);

      await managerWithRefresh.getValidToken();

      expect(listener).toHaveBeenCalledWith(newTokens);

      managerWithRefresh.dispose();
    });

    it('リスナーを解除できること', async () => {
      const oldTokens = createTestTokenPair(30);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const newTokens = createTestTokenPair(3600);
      const refresher = vi.fn().mockResolvedValue(newTokens);

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000,
        autoRefresh: false,
      });

      const listener = vi.fn();
      const unsubscribe = managerWithRefresh.onRefresh(listener);

      unsubscribe();

      await managerWithRefresh.getValidToken();

      expect(listener).not.toHaveBeenCalled();

      managerWithRefresh.dispose();
    });
  });

  describe('重複リフレッシュの防止', () => {
    it('同時に複数のリフレッシュリクエストを送っても1回だけ実行されること', async () => {
      const oldTokens = createTestTokenPair(30);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const newTokens = createTestTokenPair(3600);
      const refresher = vi.fn().mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(newTokens), 100))
      );

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000,
        autoRefresh: false,
      });

      // 同時に3つのリクエスト
      const promise1 = managerWithRefresh.getValidToken();
      const promise2 = managerWithRefresh.getValidToken();
      const promise3 = managerWithRefresh.getValidToken();

      vi.advanceTimersByTime(100);

      const results = await Promise.all([promise1, promise2, promise3]);

      // リフレッシャーは1回だけ呼ばれる
      expect(refresher).toHaveBeenCalledTimes(1);

      // 全ての結果が同じ
      expect(results.every((r) => r.type === 'refreshed')).toBe(true);

      managerWithRefresh.dispose();
    });
  });

  describe('onRefreshError コールバック', () => {
    it('リフレッシュ失敗時にエラーコールバックが呼ばれること', async () => {
      const oldTokens = createTestTokenPair(30);
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const refresher = vi.fn().mockRejectedValue(new Error('Refresh failed'));
      const onRefreshError = vi.fn();

      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
        refreshMarginMs: 60000,
        autoRefresh: false,
        onRefreshError,
      });

      await managerWithRefresh.getValidToken();

      expect(onRefreshError).toHaveBeenCalled();
      expect(onRefreshError.mock.calls[0][0].code).toBe('REFRESH_FAILED');

      managerWithRefresh.dispose();
    });
  });
});
