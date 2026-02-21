import { vi, describe, it, expect, beforeEach } from 'vitest';
import {
  parseSpiffeId,
  validateSpiffeId,
  isExpired,
  shouldRefresh,
  bearerHeader,
  HttpServiceAuthClient,
  ServiceAuthError,
  type ServiceToken,
} from '../src/index.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function mockTokenResponse(expiresIn = 3600) {
  return Promise.resolve({
    ok: true,
    status: 200,
    json: () =>
      Promise.resolve({
        access_token: 'test-token',
        token_type: 'Bearer',
        expires_in: expiresIn,
      }),
    text: () =>
      Promise.resolve(
        JSON.stringify({
          access_token: 'test-token',
          token_type: 'Bearer',
          expires_in: expiresIn,
        }),
      ),
  } as Response);
}

describe('parseSpiffeId', () => {
  it('正しくパースする', () => {
    const result = parseSpiffeId('spiffe://example.com/ns/default/sa/my-service');
    expect(result.trustDomain).toBe('example.com');
    expect(result.namespace).toBe('default');
    expect(result.serviceAccount).toBe('my-service');
    expect(result.uri).toBe('spiffe://example.com/ns/default/sa/my-service');
  });

  it('スキームがspiffeでない場合にエラーを投げる', () => {
    expect(() => parseSpiffeId('https://example.com/ns/default/sa/svc')).toThrow(
      ServiceAuthError,
    );
    expect(() => parseSpiffeId('https://example.com/ns/default/sa/svc')).toThrow(
      'must start with spiffe://',
    );
  });

  it('パスが不在の場合にエラーを投げる', () => {
    expect(() => parseSpiffeId('spiffe://example.com')).toThrow(ServiceAuthError);
  });

  it('/ns/が不在の場合にエラーを投げる', () => {
    expect(() => parseSpiffeId('spiffe://example.com/wrong/default/sa/svc')).toThrow(
      ServiceAuthError,
    );
  });
});

describe('validateSpiffeId', () => {
  it('正しいnamespaceを通す', () => {
    const result = validateSpiffeId('spiffe://example.com/ns/production/sa/svc', 'production');
    expect(result.namespace).toBe('production');
  });

  it('間違ったnamespaceでエラーを投げる', () => {
    expect(() =>
      validateSpiffeId('spiffe://example.com/ns/staging/sa/svc', 'production'),
    ).toThrow(ServiceAuthError);
    expect(() =>
      validateSpiffeId('spiffe://example.com/ns/staging/sa/svc', 'production'),
    ).toThrow('namespace mismatch');
  });
});

describe('isExpired', () => {
  it('期限切れトークンでtrueを返す', () => {
    const token: ServiceToken = {
      accessToken: 'token',
      tokenType: 'Bearer',
      expiresAt: new Date(Date.now() - 1000),
    };
    expect(isExpired(token)).toBe(true);
  });

  it('有効なトークンでfalseを返す', () => {
    const token: ServiceToken = {
      accessToken: 'token',
      tokenType: 'Bearer',
      expiresAt: new Date(Date.now() + 3600_000),
    };
    expect(isExpired(token)).toBe(false);
  });
});

describe('shouldRefresh', () => {
  it('30秒以内に期限切れのトークンでtrueを返す', () => {
    const token: ServiceToken = {
      accessToken: 'token',
      tokenType: 'Bearer',
      expiresAt: new Date(Date.now() + 10_000),
    };
    expect(shouldRefresh(token)).toBe(true);
  });

  it('十分な残余期限があるトークンでfalseを返す', () => {
    const token: ServiceToken = {
      accessToken: 'token',
      tokenType: 'Bearer',
      expiresAt: new Date(Date.now() + 3600_000),
    };
    expect(shouldRefresh(token)).toBe(false);
  });
});

describe('bearerHeader', () => {
  it('正しいヘッダー文字列を返す', () => {
    const token: ServiceToken = {
      accessToken: 'my-access-token',
      tokenType: 'Bearer',
      expiresAt: new Date(Date.now() + 3600_000),
    };
    expect(bearerHeader(token)).toBe('Bearer my-access-token');
  });
});

describe('ServiceAuthError', () => {
  it('正しく生成される', () => {
    const err = new ServiceAuthError('test error');
    expect(err.message).toBe('test error');
    expect(err.name).toBe('ServiceAuthError');
  });

  it('causeを持てる', () => {
    const cause = new Error('root cause');
    const err = new ServiceAuthError('wrapped', cause);
    expect(err.cause).toBe(cause);
  });
});

describe('HttpServiceAuthClient', () => {
  let client: HttpServiceAuthClient;

  beforeEach(() => {
    mockFetch.mockReset();
    client = new HttpServiceAuthClient({
      tokenEndpoint: 'http://localhost:8080/token',
      clientId: 'test-client',
      clientSecret: 'test-secret',
    });
  });

  describe('getToken', () => {
    it('fetchを呼んでトークンを返す', async () => {
      mockFetch.mockReturnValueOnce(mockTokenResponse());

      const token = await client.getToken();
      expect(token.accessToken).toBe('test-token');
      expect(token.tokenType).toBe('Bearer');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/token',
        expect.objectContaining({ method: 'POST' }),
      );
    });

    it('エラーレスポンスでServiceAuthErrorを投げる', async () => {
      mockFetch.mockReturnValueOnce(
        Promise.resolve({
          ok: false,
          status: 401,
          text: () => Promise.resolve('unauthorized'),
        } as Response),
      );

      await expect(client.getToken()).rejects.toThrow(ServiceAuthError);
    });
  });

  describe('getCachedToken', () => {
    it('キャッシュを使う', async () => {
      mockFetch.mockReturnValueOnce(mockTokenResponse(3600));

      const first = await client.getCachedToken();
      const second = await client.getCachedToken();

      expect(first).toBe('Bearer test-token');
      expect(second).toBe('Bearer test-token');
      expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    it('shouldRefresh時に再取得する', async () => {
      // 最初のトークンは5秒で期限切れ（30秒以内なのでrefresh対象）
      mockFetch.mockReturnValueOnce(mockTokenResponse(5));
      const first = await client.getCachedToken();
      expect(first).toBe('Bearer test-token');

      // 2回目のリクエストでは再取得される
      mockFetch.mockReturnValueOnce(mockTokenResponse(3600));
      const second = await client.getCachedToken();
      expect(second).toBe('Bearer test-token');
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });
  });

  describe('validateSpiffeId', () => {
    it('正しく動作する', () => {
      const result = client.validateSpiffeId(
        'spiffe://example.com/ns/default/sa/svc',
        'default',
      );
      expect(result.namespace).toBe('default');
    });

    it('不正なnamespaceでエラーを投げる', () => {
      expect(() =>
        client.validateSpiffeId('spiffe://example.com/ns/staging/sa/svc', 'production'),
      ).toThrow(ServiceAuthError);
    });
  });
});
