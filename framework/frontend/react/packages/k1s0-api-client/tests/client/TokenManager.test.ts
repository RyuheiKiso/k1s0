/**
 * TokenManager のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  TokenManager,
  SessionTokenStorage,
  LocalTokenStorage,
} from '../../src/auth/TokenManager';
import type { TokenPair, TokenStorage } from '../../src/auth/types';

// sessionStorage/localStorage のモック
const createStorageMock = () => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: () => {
      store = {};
    },
  };
};

describe('SessionTokenStorage', () => {
  let storage: SessionTokenStorage;
  let mockSessionStorage: ReturnType<typeof createStorageMock>;

  beforeEach(() => {
    mockSessionStorage = createStorageMock();
    Object.defineProperty(global, 'window', {
      value: {},
      writable: true,
    });
    Object.defineProperty(global, 'sessionStorage', {
      value: mockSessionStorage,
      writable: true,
    });
    storage = new SessionTokenStorage();
  });

  afterEach(() => {
    mockSessionStorage.clear();
  });

  describe('get', () => {
    it('保存されたトークンを取得できること', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000,
      };
      mockSessionStorage.setItem('k1s0_auth_tokens', JSON.stringify(tokens));

      const result = storage.get();

      expect(result).toEqual(tokens);
    });

    it('トークンがない場合は null を返すこと', () => {
      const result = storage.get();
      expect(result).toBeNull();
    });

    it('不正な JSON の場合は null を返すこと', () => {
      mockSessionStorage.setItem('k1s0_auth_tokens', 'invalid-json');

      const result = storage.get();

      expect(result).toBeNull();
    });
  });

  describe('set', () => {
    it('トークンを保存できること', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000,
      };

      storage.set(tokens);

      expect(mockSessionStorage.setItem).toHaveBeenCalledWith(
        'k1s0_auth_tokens',
        JSON.stringify(tokens)
      );
    });
  });

  describe('clear', () => {
    it('トークンを削除できること', () => {
      storage.clear();

      expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('k1s0_auth_tokens');
    });
  });
});

describe('LocalTokenStorage', () => {
  let storage: LocalTokenStorage;
  let mockLocalStorage: ReturnType<typeof createStorageMock>;

  beforeEach(() => {
    mockLocalStorage = createStorageMock();
    Object.defineProperty(global, 'window', {
      value: {},
      writable: true,
    });
    Object.defineProperty(global, 'localStorage', {
      value: mockLocalStorage,
      writable: true,
    });
    storage = new LocalTokenStorage();
  });

  afterEach(() => {
    mockLocalStorage.clear();
  });

  it('localStorage を使用してトークンを保存できること', () => {
    const tokens: TokenPair = {
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    };

    storage.set(tokens);

    expect(mockLocalStorage.setItem).toHaveBeenCalledWith(
      'k1s0_auth_tokens',
      JSON.stringify(tokens)
    );
  });
});

describe('TokenManager', () => {
  let tokenManager: TokenManager;
  let mockStorage: TokenStorage;

  beforeEach(() => {
    vi.useFakeTimers();
    mockStorage = {
      get: vi.fn(),
      set: vi.fn(),
      clear: vi.fn(),
    };
    tokenManager = new TokenManager({ storage: mockStorage });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('setTokens', () => {
    it('トークンを保存できること', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000,
      };

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
    it('トークンを取得できること', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
      };
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      const result = tokenManager.getTokens();

      expect(result).toEqual(tokens);
    });
  });

  describe('isTokenValid', () => {
    it('有効期限内のトークンは true を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000, // 1時間後
      };

      const result = tokenManager.isTokenValid(tokens);

      expect(result).toBe(true);
    });

    it('有効期限切れのトークンは false を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() - 1000, // 過去
      };

      const result = tokenManager.isTokenValid(tokens);

      expect(result).toBe(false);
    });

    it('マージン時間を考慮して判定すること', () => {
      // マージン時間を考慮（デフォルト60秒）
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 30000, // 30秒後
      };

      const result = tokenManager.isTokenValid(tokens);

      // 60秒のマージンがあるので、30秒後は無効と判定される
      expect(result).toBe(false);
    });

    it('expiresAt がない場合は true を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
      };

      const result = tokenManager.isTokenValid(tokens);

      expect(result).toBe(true);
    });
  });

  describe('needsRefresh', () => {
    it('リフレッシュが必要な場合 true を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 30000, // 30秒後
      };

      const result = tokenManager.needsRefresh(tokens);

      // 60秒のマージン内なのでリフレッシュが必要
      expect(result).toBe(true);
    });

    it('リフレッシュが不要な場合 false を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000, // 1時間後
      };

      const result = tokenManager.needsRefresh(tokens);

      expect(result).toBe(false);
    });

    it('refreshToken がない場合は false を返すこと', () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        expiresAt: Date.now() + 30000,
      };

      const result = tokenManager.needsRefresh(tokens);

      expect(result).toBe(false);
    });
  });

  describe('getValidToken', () => {
    it('トークンがない場合は none を返すこと', async () => {
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(null);

      const result = await tokenManager.getValidToken();

      expect(result).toEqual({ type: 'none' });
    });

    it('有効なトークンがある場合は valid を返すこと', async () => {
      const tokens: TokenPair = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 3600000, // 1時間後
      };
      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(tokens);

      const result = await tokenManager.getValidToken();

      expect(result).toEqual({ type: 'valid', token: 'access-token' });
    });

    it('リフレッシュが成功した場合は refreshed を返すこと', async () => {
      const oldTokens: TokenPair = {
        accessToken: 'old-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 30000, // マージン内
      };
      const newTokens: TokenPair = {
        accessToken: 'new-token',
        refreshToken: 'new-refresh-token',
        expiresAt: Date.now() + 3600000,
      };

      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const refresher = vi.fn().mockResolvedValue(newTokens);
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
      });

      const result = await managerWithRefresh.getValidToken();

      expect(result).toEqual({ type: 'refreshed', token: 'new-token' });
      expect(mockStorage.set).toHaveBeenCalledWith(newTokens);
    });

    it('リフレッシュが失敗した場合は expired を返すこと', async () => {
      const oldTokens: TokenPair = {
        accessToken: 'old-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 30000, // マージン内
      };

      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const refresher = vi.fn().mockResolvedValue(null);
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
      });

      const result = await managerWithRefresh.getValidToken();

      expect(result).toEqual({ type: 'expired' });
    });

    it('重複するリフレッシュリクエストが防止されること', async () => {
      const oldTokens: TokenPair = {
        accessToken: 'old-token',
        refreshToken: 'refresh-token',
        expiresAt: Date.now() + 30000,
      };
      const newTokens: TokenPair = {
        accessToken: 'new-token',
        refreshToken: 'new-refresh-token',
        expiresAt: Date.now() + 3600000,
      };

      (mockStorage.get as ReturnType<typeof vi.fn>).mockReturnValue(oldTokens);

      const refresher = vi.fn().mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve(newTokens), 100))
      );
      const managerWithRefresh = new TokenManager({
        storage: mockStorage,
        refreshToken: refresher,
      });

      // 同時に複数のリクエスト
      const promise1 = managerWithRefresh.getValidToken();
      const promise2 = managerWithRefresh.getValidToken();

      vi.advanceTimersByTime(100);

      const [result1, result2] = await Promise.all([promise1, promise2]);

      // リフレッシャーは1回だけ呼ばれる
      expect(refresher).toHaveBeenCalledTimes(1);
      expect(result1.type).toBe('refreshed');
      expect(result2.type).toBe('refreshed');
    });
  });
});
