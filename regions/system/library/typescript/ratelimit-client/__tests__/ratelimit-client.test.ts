import { describe, it, expect } from 'vitest';
import { InMemoryRateLimitClient, RateLimitError } from '../src/index.js';
import type { RateLimitPolicy } from '../src/index.js';

describe('InMemoryRateLimitClient', () => {
  it('checkで許可を返す', async () => {
    const client = new InMemoryRateLimitClient();
    const status = await client.check('test-key', 1);
    expect(status.allowed).toBe(true);
    expect(status.remaining).toBe(99);
    expect(status.retryAfterSecs).toBeUndefined();
  });

  it('checkで制限超過を返す', async () => {
    const client = new InMemoryRateLimitClient();
    const policy: RateLimitPolicy = {
      key: 'limited',
      limit: 2,
      windowSecs: 60,
      algorithm: 'fixed_window',
    };
    client.setPolicy('limited', policy);

    await client.consume('limited', 2);
    const status = await client.check('limited', 1);
    expect(status.allowed).toBe(false);
    expect(status.remaining).toBe(0);
    expect(status.retryAfterSecs).toBe(60);
  });

  it('consumeで使用量を消費する', async () => {
    const client = new InMemoryRateLimitClient();
    const result = await client.consume('test-key', 1);
    expect(result.remaining).toBe(99);
    expect(client.getUsedCount('test-key')).toBe(1);
  });

  it('consumeで制限超過時にエラーを投げる', async () => {
    const client = new InMemoryRateLimitClient();
    client.setPolicy('small', {
      key: 'small',
      limit: 1,
      windowSecs: 60,
      algorithm: 'token_bucket',
    });

    await client.consume('small', 1);
    await expect(client.consume('small', 1)).rejects.toThrow(RateLimitError);
  });

  it('getLimitでデフォルトポリシーを返す', async () => {
    const client = new InMemoryRateLimitClient();
    const policy = await client.getLimit('unknown');
    expect(policy.limit).toBe(100);
    expect(policy.windowSecs).toBe(3600);
    expect(policy.algorithm).toBe('token_bucket');
  });

  it('getLimitでカスタムポリシーを返す', async () => {
    const client = new InMemoryRateLimitClient();
    client.setPolicy('tenant:T1', {
      key: 'tenant:T1',
      limit: 50,
      windowSecs: 1800,
      algorithm: 'sliding_window',
    });

    const policy = await client.getLimit('tenant:T1');
    expect(policy.key).toBe('tenant:T1');
    expect(policy.limit).toBe(50);
    expect(policy.algorithm).toBe('sliding_window');
  });

  it('RateLimitErrorにコードが含まれる', () => {
    const err = new RateLimitError('exceeded', 'LIMIT_EXCEEDED', 30);
    expect(err.code).toBe('LIMIT_EXCEEDED');
    expect(err.retryAfterSecs).toBe(30);
    expect(err.name).toBe('RateLimitError');
  });

  it('複数コストのcheckが正しく動作する', async () => {
    const client = new InMemoryRateLimitClient();
    client.setPolicy('cost-key', {
      key: 'cost-key',
      limit: 10,
      windowSecs: 60,
      algorithm: 'fixed_window',
    });

    const status1 = await client.check('cost-key', 5);
    expect(status1.allowed).toBe(true);
    expect(status1.remaining).toBe(5);

    const status2 = await client.check('cost-key', 11);
    expect(status2.allowed).toBe(false);
  });
});
