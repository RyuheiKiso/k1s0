import { describe, it, expect } from 'vitest';
import { Bulkhead, BulkheadFullError } from '../src/index.js';

describe('Bulkhead', () => {
  it('acquire and release', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 2, maxWaitDurationMs: 100 });

    await bulkhead.acquire();
    await bulkhead.acquire();
    bulkhead.release();
    bulkhead.release();
  });

  it('rejects when full and timeout', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 1, maxWaitDurationMs: 50 });

    await bulkhead.acquire();

    await expect(bulkhead.acquire()).rejects.toThrow(BulkheadFullError);

    bulkhead.release();
  });

  it('waits for slot release', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 1, maxWaitDurationMs: 500 });

    await bulkhead.acquire();

    const waiting = bulkhead.acquire();

    setTimeout(() => bulkhead.release(), 30);

    await expect(waiting).resolves.toBeUndefined();

    bulkhead.release();
  });

  it('call succeeds', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 1, maxWaitDurationMs: 100 });

    const result = await bulkhead.call(async () => 42);

    expect(result).toBe(42);
  });

  it('call rejects when full', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 1, maxWaitDurationMs: 50 });

    await bulkhead.acquire();

    await expect(bulkhead.call(async () => 42)).rejects.toThrow(BulkheadFullError);

    bulkhead.release();
  });

  it('concurrent access respects limit', async () => {
    const bulkhead = new Bulkhead({ maxConcurrentCalls: 2, maxWaitDurationMs: 500 });
    let concurrent = 0;
    let maxConcurrent = 0;

    const task = async () => {
      await bulkhead.call(async () => {
        concurrent++;
        maxConcurrent = Math.max(maxConcurrent, concurrent);
        await new Promise((r) => setTimeout(r, 30));
        concurrent--;
      });
    };

    await Promise.all([task(), task(), task(), task()]);

    expect(maxConcurrent).toBeLessThanOrEqual(2);
    expect(maxConcurrent).toBe(2);
  });
});
