// @k1s0/api-client の単体テスト。

import { describe, it, expect, vi } from 'vitest';
import { ApiClient, ApiError } from '../index';
import { stubConfig } from '@k1s0/config';

// 仮の fetch 実装を組み立てるヘルパ。
function mockFetch(impl: (url: string, init: RequestInit) => Promise<Response>): typeof fetch {
  return vi.fn(impl) as unknown as typeof fetch;
}

describe('ApiClient.stateGet', () => {
  it('成功時に StateValue を返す', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/state/get');
      expect(init.method).toBe('POST');
      expect(init.headers).toMatchObject({
        'Content-Type': 'application/json',
        'X-Tenant-Id': 'tenant-test',
      });
      const body = JSON.parse(init.body as string);
      expect(body.store).toBe('postgres');
      expect(body.key).toBe('user/123');
      return new Response(
        JSON.stringify({ data: '{"name":"alice"}', etag: 'abc', found: true }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      );
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    const value = await client.stateGet('postgres', 'user/123');
    expect(value.found).toBe(true);
    expect(value.etag).toBe('abc');
  });

  it('エラー時に ApiError を投げる', async () => {
    const fetchFn = mockFetch(async () => {
      return new Response(
        JSON.stringify({ error: { code: 'E-T3-BFF-AUTH-001', message: 'missing bearer token', category: 'UNAUTHORIZED' } }),
        { status: 401, headers: { 'Content-Type': 'application/json' } },
      );
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await expect(client.stateGet('postgres', 'user/123')).rejects.toBeInstanceOf(ApiError);
  });

  it('Authorization ヘッダを getToken から付与する', async () => {
    const fetchFn = mockFetch(async (_url, init) => {
      expect((init.headers as Record<string, string>)['Authorization']).toBe('Bearer abc-token');
      return new Response('{"data":"","etag":"","found":false}', { status: 200 });
    });
    const client = new ApiClient({
      config: stubConfig(),
      fetchFn,
      getToken: () => 'abc-token',
    });
    await client.stateGet('postgres', 'k');
  });
});

describe('ApiClient.graphql', () => {
  it('errors 配列があれば ApiError を投げる', async () => {
    const fetchFn = mockFetch(async () =>
      new Response(JSON.stringify({ errors: [{ message: 'unsupported query' }] }), { status: 200 }),
    );
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await expect(client.graphql('query { foo }')).rejects.toBeInstanceOf(ApiError);
  });
});
