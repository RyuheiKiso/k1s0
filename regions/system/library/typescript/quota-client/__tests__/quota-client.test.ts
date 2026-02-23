import { describe, it, expect, beforeEach } from 'vitest';
import {
  InMemoryQuotaClient,
  CachedQuotaClient,
  QuotaExceededError,
} from '../src/index';

describe('InMemoryQuotaClient', () => {
  let client: InMemoryQuotaClient;

  beforeEach(() => {
    client = new InMemoryQuotaClient();
  });

  it('check returns allowed for within-limit request', async () => {
    const status = await client.check('q1', 100);
    expect(status.allowed).toBe(true);
    expect(status.remaining).toBe(1000);
    expect(status.limit).toBe(1000);
  });

  it('check returns not allowed when exceeded', async () => {
    await client.increment('q1', 900);
    const status = await client.check('q1', 200);
    expect(status.allowed).toBe(false);
    expect(status.remaining).toBe(100);
  });

  it('increment accumulates usage', async () => {
    await client.increment('q1', 300);
    const usage = await client.increment('q1', 200);
    expect(usage.used).toBe(500);
    expect(usage.limit).toBe(1000);
  });

  it('getUsage returns current usage', async () => {
    await client.increment('q1', 100);
    const usage = await client.getUsage('q1');
    expect(usage.used).toBe(100);
    expect(usage.quotaId).toBe('q1');
  });

  it('getPolicy returns default policy', async () => {
    const policy = await client.getPolicy('q1');
    expect(policy.quotaId).toBe('q1');
    expect(policy.limit).toBe(1000);
    expect(policy.period).toBe('daily');
    expect(policy.resetStrategy).toBe('fixed');
  });

  it('getPolicy returns custom policy', async () => {
    client.setPolicy('q1', {
      quotaId: 'q1',
      limit: 5000,
      period: 'monthly',
      resetStrategy: 'sliding',
    });
    const policy = await client.getPolicy('q1');
    expect(policy.limit).toBe(5000);
    expect(policy.period).toBe('monthly');
  });
});

describe('CachedQuotaClient', () => {
  it('caches policy within TTL', async () => {
    const inner = new InMemoryQuotaClient();
    const cached = new CachedQuotaClient(inner, 60000);

    const p1 = await cached.getPolicy('q1');
    inner.setPolicy('q1', {
      quotaId: 'q1',
      limit: 9999,
      period: 'hourly',
      resetStrategy: 'fixed',
    });
    const p2 = await cached.getPolicy('q1');
    expect(p2.limit).toBe(p1.limit);
  });

  it('delegates check to inner', async () => {
    const inner = new InMemoryQuotaClient();
    const cached = new CachedQuotaClient(inner, 60000);
    const status = await cached.check('q1', 100);
    expect(status.allowed).toBe(true);
  });

  it('delegates increment to inner', async () => {
    const inner = new InMemoryQuotaClient();
    const cached = new CachedQuotaClient(inner, 60000);
    const usage = await cached.increment('q1', 100);
    expect(usage.used).toBe(100);
  });

  it('delegates getUsage to inner', async () => {
    const inner = new InMemoryQuotaClient();
    const cached = new CachedQuotaClient(inner, 60000);
    await cached.increment('q1', 50);
    const usage = await cached.getUsage('q1');
    expect(usage.used).toBe(50);
  });
});

describe('QuotaExceededError', () => {
  it('includes quotaId and remaining', () => {
    const err = new QuotaExceededError('q1', 0);
    expect(err.quotaId).toBe('q1');
    expect(err.remaining).toBe(0);
    expect(err.name).toBe('QuotaExceededError');
  });
});
