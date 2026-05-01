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

describe('ApiClient state save / delete', () => {
  it('stateSave で /api/state/save に POST する', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/state/save');
      const body = JSON.parse(init.body as string);
      expect(body).toEqual({ store: 's', key: 'k', data: 'payload' });
      return new Response(JSON.stringify({ etag: 'v9' }), { status: 200 });
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    const got = await client.stateSave('s', 'k', 'payload');
    expect(got.etag).toBe('v9');
  });

  it('stateDelete は expected_etag を任意で付与する', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/state/delete');
      const body = JSON.parse(init.body as string);
      expect(body).toEqual({ store: 's', key: 'k', expected_etag: 'e1' });
      return new Response('{}', { status: 200 });
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await client.stateDelete('s', 'k', 'e1');
  });

  it('stateDelete は expectedEtag 未指定なら expected_etag を付けない', async () => {
    const fetchFn = mockFetch(async (_url, init) => {
      const body = JSON.parse(init.body as string);
      expect(body).toEqual({ store: 's', key: 'k' });
      return new Response('{}', { status: 200 });
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await client.stateDelete('s', 'k');
  });
});

describe('ApiClient new service endpoints', () => {
  it('auditRecord で /api/audit/record に POST し audit_id を取得する', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/audit/record');
      const body = JSON.parse(init.body as string);
      expect(body.actor).toBe('alice');
      expect(body.action).toBe('READ');
      return new Response(JSON.stringify({ audit_id: 'aud-123' }), { status: 200 });
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    const got = await client.auditRecord({
      actor: 'alice',
      action: 'READ',
      resource: 'user/1',
      outcome: 'SUCCESS',
    });
    expect(got.audit_id).toBe('aud-123');
  });

  it('featureEvaluateBoolean で /api/feature/evaluate-boolean に POST する', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/feature/evaluate-boolean');
      const body = JSON.parse(init.body as string);
      expect(body).toEqual({ flag_key: 'new-ui', eval_ctx: { tenant: 't1' } });
      return new Response(
        JSON.stringify({ value: true, variant: 'on', reason: 'TARGETING_MATCH' }),
        { status: 200 },
      );
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    const got = await client.featureEvaluateBoolean({
      flag_key: 'new-ui',
      eval_ctx: { tenant: 't1' },
    });
    expect(got.value).toBe(true);
    expect(got.variant).toBe('on');
    expect(got.reason).toBe('TARGETING_MATCH');
  });

  it('telemetryEmitMetric で points を /api/telemetry/emit-metric に詰めて POST する', async () => {
    const fetchFn = mockFetch(async (url, init) => {
      expect(url).toBe('http://localhost:8080/api/telemetry/emit-metric');
      const body = JSON.parse(init.body as string);
      expect(body).toEqual({
        points: [
          { name: 'k1s0.web.requests_total', value: 1, labels: { route: '/' } },
        ],
      });
      return new Response('{}', { status: 200 });
    });
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await client.telemetryEmitMetric([
      { name: 'k1s0.web.requests_total', value: 1, labels: { route: '/' } },
    ]);
  });

  it('upstream エラーは ApiError を投げる', async () => {
    const fetchFn = mockFetch(async () =>
      new Response(
        JSON.stringify({ code: 'E-T3-BFF-PII-200', message: 'pii classify failed' }),
        { status: 502 },
      ),
    );
    const client = new ApiClient({ config: stubConfig(), fetchFn });
    await expect(client.piiClassify('hello')).rejects.toBeInstanceOf(ApiError);
  });
});
