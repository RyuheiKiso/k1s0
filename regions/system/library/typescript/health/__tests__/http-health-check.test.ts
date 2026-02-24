import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { HttpHealthCheck } from '../src/http-health-check.js';

describe('HttpHealthCheck', () => {
  const originalFetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it('デフォルト名が"http"であること', () => {
    const check = new HttpHealthCheck({ url: 'http://example.com/healthz' });
    expect(check.name).toBe('http');
  });

  it('カスタム名を設定できること', () => {
    const check = new HttpHealthCheck({
      url: 'http://example.com/healthz',
      name: 'upstream',
    });
    expect(check.name).toBe('upstream');
  });

  it('2xxレスポンスでhealthyとなること', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
    } as Response);

    const check = new HttpHealthCheck({ url: 'http://example.com/healthz' });
    await expect(check.check()).resolves.toBeUndefined();
  });

  it('非2xxレスポンスでエラーを投げること', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 503,
    } as Response);

    const check = new HttpHealthCheck({ url: 'http://example.com/healthz' });
    await expect(check.check()).rejects.toThrow('status 503');
  });

  it('ネットワークエラーでエラーを投げること', async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new Error('fetch failed'));

    const check = new HttpHealthCheck({ url: 'http://example.com/healthz' });
    await expect(check.check()).rejects.toThrow('fetch failed');
  });

  it('タイムアウトでエラーを投げること', async () => {
    const abortError = new DOMException('signal is aborted', 'AbortError');
    globalThis.fetch = vi.fn().mockRejectedValue(abortError);

    const check = new HttpHealthCheck({
      url: 'http://example.com/healthz',
      timeoutMs: 100,
    });
    await expect(check.check()).rejects.toThrow('timeout');
  });
});
