import { describe, it, expect, vi } from 'vitest';
import {
  generateSignature,
  verifySignature,
  InMemoryWebhookClient,
  HttpWebhookClient,
  WebhookError,
} from '../src/index.js';
import type { WebhookPayload } from '../src/index.js';

describe('generateSignature', () => {
  it('HMAC-SHA256のhex文字列を返す', () => {
    const sig = generateSignature('secret', '{"data":1}');
    expect(sig).toMatch(/^[0-9a-f]{64}$/);
  });

  it('同じ入力で同じ署名を返す', () => {
    const s1 = generateSignature('key', 'body');
    const s2 = generateSignature('key', 'body');
    expect(s1).toBe(s2);
  });

  it('異なるsecretで異なる署名を返す', () => {
    const s1 = generateSignature('key1', 'body');
    const s2 = generateSignature('key2', 'body');
    expect(s1).not.toBe(s2);
  });
});

describe('verifySignature', () => {
  it('正しい署名で検証成功する', () => {
    const body = '{"event":"test"}';
    const sig = generateSignature('secret', body);
    expect(verifySignature('secret', body, sig)).toBe(true);
  });

  it('不正な署名で検証失敗する', () => {
    const body = '{"event":"test"}';
    const sig = generateSignature('secret', body);
    expect(verifySignature('wrong-secret', body, sig)).toBe(false);
  });
});

describe('InMemoryWebhookClient', () => {
  it('送信ペイロードを記録する', async () => {
    const client = new InMemoryWebhookClient();
    const payload: WebhookPayload = {
      eventType: 'user.created',
      timestamp: new Date().toISOString(),
      data: { userId: '123' },
    };
    const status = await client.send('https://example.com/webhook', payload);
    expect(status).toBe(200);
    expect(client.getSent()).toHaveLength(1);
    expect(client.getSent()[0].payload.eventType).toBe('user.created');
  });
});

describe('HttpWebhookClient', () => {
  const testPayload: WebhookPayload = {
    eventType: 'user.created',
    timestamp: '2026-01-01T00:00:00Z',
    data: { userId: '123' },
  };

  function createMockFetch(responses: Array<{ status: number }>): typeof fetch {
    let callIndex = 0;
    return vi.fn(async () => {
      const resp = responses[callIndex] ?? responses[responses.length - 1];
      callIndex++;
      return new Response(null, { status: resp.status });
    }) as unknown as typeof fetch;
  }

  it('成功時にステータスコードを返す', async () => {
    const mockFetch = createMockFetch([{ status: 200 }]);
    const client = new HttpWebhookClient({ secret: 'test-secret' }, mockFetch);

    const status = await client.send('https://example.com/hook', testPayload);

    expect(status).toBe(200);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('Idempotency-Keyヘッダーを付与する', async () => {
    const mockFetch = vi.fn(async () => new Response(null, { status: 200 })) as unknown as typeof fetch;
    const client = new HttpWebhookClient({}, mockFetch);

    await client.send('https://example.com/hook', testPayload);

    const calledWith = (mockFetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const headers = calledWith[1].headers as Record<string, string>;
    expect(headers['Idempotency-Key']).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );
  });

  it('secretが設定されている場合X-K1s0-Signatureヘッダーを付与する', async () => {
    const mockFetch = vi.fn(async () => new Response(null, { status: 200 })) as unknown as typeof fetch;
    const client = new HttpWebhookClient({ secret: 'my-secret' }, mockFetch);

    await client.send('https://example.com/hook', testPayload);

    const calledWith = (mockFetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const headers = calledWith[1].headers as Record<string, string>;
    const body = calledWith[1].body as string;
    const expectedSig = generateSignature('my-secret', body);
    expect(headers['X-K1s0-Signature']).toBe(expectedSig);
  });

  it('secretが未設定の場合X-K1s0-Signatureヘッダーを付与しない', async () => {
    const mockFetch = vi.fn(async () => new Response(null, { status: 200 })) as unknown as typeof fetch;
    const client = new HttpWebhookClient({}, mockFetch);

    await client.send('https://example.com/hook', testPayload);

    const calledWith = (mockFetch as ReturnType<typeof vi.fn>).mock.calls[0];
    const headers = calledWith[1].headers as Record<string, string>;
    expect(headers['X-K1s0-Signature']).toBeUndefined();
  });

  it('5xxレスポンスでリトライする', async () => {
    const mockFetch = createMockFetch([
      { status: 500 },
      { status: 503 },
      { status: 200 },
    ]);
    const client = new HttpWebhookClient(
      { maxRetries: 3, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    const status = await client.send('https://example.com/hook', testPayload);

    expect(status).toBe(200);
    expect(mockFetch).toHaveBeenCalledTimes(3);
  });

  it('429レスポンスでリトライする', async () => {
    const mockFetch = createMockFetch([
      { status: 429 },
      { status: 200 },
    ]);
    const client = new HttpWebhookClient(
      { maxRetries: 3, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    const status = await client.send('https://example.com/hook', testPayload);

    expect(status).toBe(200);
    expect(mockFetch).toHaveBeenCalledTimes(2);
  });

  it('4xxレスポンス（429以外）ではリトライしない', async () => {
    const mockFetch = createMockFetch([{ status: 400 }]);
    const client = new HttpWebhookClient(
      { maxRetries: 3, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    const status = await client.send('https://example.com/hook', testPayload);

    expect(status).toBe(400);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('リトライ上限超過でMAX_RETRIES_EXCEEDEDエラーを投げる', async () => {
    const mockFetch = createMockFetch([
      { status: 500 },
      { status: 500 },
      { status: 500 },
      { status: 500 },
    ]);
    const client = new HttpWebhookClient(
      { maxRetries: 2, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    await expect(
      client.send('https://example.com/hook', testPayload),
    ).rejects.toThrow(WebhookError);

    try {
      await client.send('https://example.com/hook', testPayload);
    } catch (err) {
      expect(err).toBeInstanceOf(WebhookError);
      expect((err as WebhookError).code).toBe('MAX_RETRIES_EXCEEDED');
    }
  });

  it('ネットワークエラーでリトライする', async () => {
    let callCount = 0;
    const mockFetch = vi.fn(async () => {
      callCount++;
      if (callCount <= 2) {
        throw new Error('Network error');
      }
      return new Response(null, { status: 200 });
    }) as unknown as typeof fetch;

    const client = new HttpWebhookClient(
      { maxRetries: 3, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    const status = await client.send('https://example.com/hook', testPayload);

    expect(status).toBe(200);
    expect(mockFetch).toHaveBeenCalledTimes(3);
  });

  it('全リトライでIdempotency-Keyが同一である', async () => {
    const mockFetch = vi.fn(async () => new Response(null, { status: 500 })) as unknown as typeof fetch;
    const client = new HttpWebhookClient(
      { maxRetries: 2, initialBackoffMs: 1, maxBackoffMs: 10 },
      mockFetch,
    );

    try {
      await client.send('https://example.com/hook', testPayload);
    } catch {
      // expected
    }

    const calls = (mockFetch as ReturnType<typeof vi.fn>).mock.calls;
    const keys = calls.map(
      (c: [string, { headers: Record<string, string> }]) => c[1].headers['Idempotency-Key'],
    );
    expect(keys.length).toBe(3);
    expect(new Set(keys).size).toBe(1);
  });
});
