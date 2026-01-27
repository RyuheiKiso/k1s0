/**
 * ApiClient のテスト
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ApiClient, createApiClient } from '../../src/client/ApiClient';
import { ApiError } from '../../src/error/ApiError';
import type { TokenManager } from '../../src/auth/TokenManager';

// fetch のモック
const mockFetch = vi.fn();
global.fetch = mockFetch;

// AbortController のモック
class MockAbortController {
  signal = {
    aborted: false,
    reason: undefined as unknown,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  };
  abort = vi.fn((reason?: unknown) => {
    this.signal.aborted = true;
    this.signal.reason = reason;
  });
}
global.AbortController = MockAbortController as unknown as typeof AbortController;

// Response のヘルパー
const createMockResponse = (
  data: unknown,
  options: { status?: number; statusText?: string; headers?: Record<string, string> } = {}
) => {
  const { status = 200, statusText = 'OK', headers = {} } = options;
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText,
    headers: {
      get: (name: string) => headers[name.toLowerCase()] ?? null,
      ...headers,
    },
    json: vi.fn().mockResolvedValue(data),
    text: vi.fn().mockResolvedValue(JSON.stringify(data)),
  };
};

describe('ApiClient', () => {
  let client: ApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    client = createApiClient({
      baseUrl: 'https://api.example.com',
      timeout: 5000,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('基本的なリクエスト', () => {
    it('GET リクエストが正しく送信されること', async () => {
      const responseData = { id: 1, name: 'Test' };
      mockFetch.mockResolvedValueOnce(createMockResponse(responseData));

      const result = await client.get<typeof responseData>('/users/1');

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users/1',
        expect.objectContaining({
          method: 'GET',
        })
      );
      expect(result.data).toEqual(responseData);
      expect(result.status).toBe(200);
    });

    it('POST リクエストが正しく送信されること', async () => {
      const requestBody = { name: 'New User', email: 'test@example.com' };
      const responseData = { id: 1, ...requestBody };
      mockFetch.mockResolvedValueOnce(createMockResponse(responseData, { status: 201 }));

      const result = await client.post<typeof responseData>('/users', requestBody);

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(requestBody),
        })
      );
      expect(result.data).toEqual(responseData);
      expect(result.status).toBe(201);
    });

    it('PUT リクエストが正しく送信されること', async () => {
      const requestBody = { name: 'Updated User' };
      const responseData = { id: 1, ...requestBody };
      mockFetch.mockResolvedValueOnce(createMockResponse(responseData));

      const result = await client.put<typeof responseData>('/users/1', requestBody);

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users/1',
        expect.objectContaining({
          method: 'PUT',
          body: JSON.stringify(requestBody),
        })
      );
      expect(result.data).toEqual(responseData);
    });

    it('PATCH リクエストが正しく送信されること', async () => {
      const requestBody = { name: 'Patched User' };
      const responseData = { id: 1, ...requestBody };
      mockFetch.mockResolvedValueOnce(createMockResponse(responseData));

      const result = await client.patch<typeof responseData>('/users/1', requestBody);

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users/1',
        expect.objectContaining({
          method: 'PATCH',
          body: JSON.stringify(requestBody),
        })
      );
      expect(result.data).toEqual(responseData);
    });

    it('DELETE リクエストが正しく送信されること', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({}, { status: 204 }));

      const result = await client.delete('/users/1');

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users/1',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
      expect(result.status).toBe(204);
    });
  });

  describe('ヘッダー', () => {
    it('デフォルトヘッダーが設定されること', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await client.get('/test');

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers['Content-Type']).toBe('application/json');
      expect(callArgs[1].headers['Accept']).toBe('application/json');
    });

    it('カスタムヘッダーが追加できること', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await client.get('/test', { headers: { 'X-Custom-Header': 'custom-value' } });

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers['X-Custom-Header']).toBe('custom-value');
    });

    it('traceparent ヘッダーが設定されること', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await client.get('/test');

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers['traceparent']).toBeDefined();
    });
  });

  describe('認証トークン', () => {
    it('TokenManager が設定されている場合、Authorization ヘッダーが追加されること', async () => {
      const mockTokenManager = {
        getValidToken: vi.fn().mockResolvedValue({ type: 'valid', token: 'test-token' }),
      } as unknown as TokenManager;

      const clientWithAuth = createApiClient({
        baseUrl: 'https://api.example.com',
        tokenManager: mockTokenManager,
      });

      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await clientWithAuth.get('/test');

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers['Authorization']).toBe('Bearer test-token');
    });

    it('skipAuth オプションで認証をスキップできること', async () => {
      const mockTokenManager = {
        getValidToken: vi.fn().mockResolvedValue({ type: 'valid', token: 'test-token' }),
      } as unknown as TokenManager;

      const clientWithAuth = createApiClient({
        baseUrl: 'https://api.example.com',
        tokenManager: mockTokenManager,
      });

      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await clientWithAuth.get('/test', { skipAuth: true });

      const callArgs = mockFetch.mock.calls[0];
      expect(callArgs[1].headers['Authorization']).toBeUndefined();
    });

    it('トークンが期限切れの場合、onAuthError が呼ばれること', async () => {
      const onAuthError = vi.fn();
      const mockTokenManager = {
        getValidToken: vi.fn().mockResolvedValue({ type: 'expired' }),
      } as unknown as TokenManager;

      const clientWithAuth = createApiClient({
        baseUrl: 'https://api.example.com',
        tokenManager: mockTokenManager,
        onAuthError,
      });

      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await clientWithAuth.get('/test');

      expect(onAuthError).toHaveBeenCalled();
    });
  });

  describe('エラーハンドリング', () => {
    it('4xx エラーが ApiError として返されること', async () => {
      const errorResponse = {
        error_code: 'NOT_FOUND',
        title: 'Not Found',
        detail: 'Resource not found',
        status: 404,
      };
      mockFetch.mockResolvedValueOnce(
        createMockResponse(errorResponse, {
          status: 404,
          headers: { 'content-type': 'application/problem+json' },
        })
      );

      await expect(client.get('/nonexistent')).rejects.toThrow(ApiError);

      try {
        await client.get('/nonexistent');
      } catch (error) {
        if (error instanceof ApiError) {
          expect(error.status).toBe(404);
          expect(error.errorCode).toBe('NOT_FOUND');
        }
      }
    });

    it('401 エラーで onAuthError が呼ばれること', async () => {
      const onAuthError = vi.fn();
      const clientWithCallback = createApiClient({
        baseUrl: 'https://api.example.com',
        onAuthError,
      });

      const errorResponse = {
        error_code: 'UNAUTHORIZED',
        title: 'Unauthorized',
        detail: 'Authentication required',
        status: 401,
      };
      mockFetch.mockResolvedValueOnce(
        createMockResponse(errorResponse, {
          status: 401,
          headers: { 'content-type': 'application/problem+json' },
        })
      );

      await expect(clientWithCallback.get('/protected')).rejects.toThrow(ApiError);
      expect(onAuthError).toHaveBeenCalled();
    });

    it('ネットワークエラーが ApiError として変換されること', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network Error'));

      await expect(client.get('/test')).rejects.toThrow(ApiError);

      try {
        await client.get('/test');
      } catch (error) {
        if (error instanceof ApiError) {
          expect(error.kind).toBe('network');
          expect(error.errorCode).toBe('NETWORK_ERROR');
        }
      }
    });
  });

  describe('204 No Content レスポンス', () => {
    it('204 レスポンスは空オブジェクトを返すこと', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse(null, { status: 204 }));

      const result = await client.delete('/users/1');

      expect(result.status).toBe(204);
      expect(result.data).toEqual({});
    });
  });

  describe('URL の構築', () => {
    it('baseUrl の末尾スラッシュが正しく処理されること', async () => {
      const clientWithSlash = createApiClient({
        baseUrl: 'https://api.example.com/',
      });

      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await clientWithSlash.get('/users');

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users',
        expect.any(Object)
      );
    });

    it('path の先頭スラッシュがない場合も正しく処理されること', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({}));

      await client.get('users');

      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/users',
        expect.any(Object)
      );
    });
  });

  describe('trace_id の取得', () => {
    it('レスポンスヘッダーから trace_id を取得できること', async () => {
      mockFetch.mockResolvedValueOnce(
        createMockResponse({}, { headers: { 'x-trace-id': 'test-trace-id' } })
      );

      const result = await client.get('/test');

      expect(result.traceId).toBe('test-trace-id');
    });
  });
});

describe('createApiClient', () => {
  it('ApiClient インスタンスを作成すること', () => {
    const client = createApiClient({ baseUrl: 'https://api.example.com' });
    expect(client).toBeInstanceOf(ApiClient);
  });
});
