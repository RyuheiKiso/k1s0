import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { RequestThrottle } from '../throttle.js';

describe('RequestThrottle', () => {
  let throttle: RequestThrottle;

  afterEach(() => {
    throttle?.dispose();
  });

  it('test_acquire_within_limit', async () => {
    throttle = new RequestThrottle({ maxRequestsPerSecond: 5, maxConcurrent: 5, maxQueueSize: 10 });

    await throttle.acquire();
    await throttle.acquire();
    await throttle.acquire();

    expect(throttle.stats.allowed).toBe(3);
    expect(throttle.stats.rejected).toBe(0);
    expect(throttle.stats.active).toBe(3);
  });

  it('test_acquire_exceeds_queue', async () => {
    throttle = new RequestThrottle({ maxRequestsPerSecond: 1, maxConcurrent: 1, maxQueueSize: 2 });

    // Use up the token and concurrency slot
    await throttle.acquire();

    // These two go into the queue (queue size = 2)
    const p1 = throttle.acquire();
    const p2 = throttle.acquire();

    // Third should be rejected (queue full)
    await expect(throttle.acquire()).rejects.toThrow('Request throttle queue full');
    expect(throttle.stats.rejected).toBe(1);

    // Clean up: release to let queued items resolve
    throttle.release();
    throttle.release();
  });

  it('test_concurrent_limit', async () => {
    throttle = new RequestThrottle({ maxRequestsPerSecond: 10, maxConcurrent: 2, maxQueueSize: 10 });

    await throttle.acquire();
    await throttle.acquire();

    // Third request should be queued (concurrency limit reached)
    const pending = throttle.acquire();
    expect(throttle.stats.queued).toBe(1);
    expect(throttle.stats.active).toBe(2);

    // Release one, pending should resolve
    throttle.release();
    await pending;
    expect(throttle.stats.active).toBe(2);
    expect(throttle.stats.queued).toBe(0);
  });

  it('test_token_refill', async () => {
    vi.useFakeTimers();

    throttle = new RequestThrottle({ maxRequestsPerSecond: 2, maxConcurrent: 10, maxQueueSize: 10 });

    await throttle.acquire();
    await throttle.acquire();

    // Tokens exhausted, next request queued
    const pending = throttle.acquire();
    expect(throttle.stats.queued).toBe(1);

    // Advance timer to trigger refill
    vi.advanceTimersByTime(1000);

    await pending;
    expect(throttle.stats.queued).toBe(0);
    expect(throttle.stats.allowed).toBe(3);

    vi.useRealTimers();
  });

  it('test_dispose_cleanup', async () => {
    throttle = new RequestThrottle({ maxRequestsPerSecond: 1, maxConcurrent: 1, maxQueueSize: 10 });

    await throttle.acquire();

    // Queue a request
    const pending = throttle.acquire();
    expect(throttle.stats.queued).toBe(1);

    throttle.dispose();
    expect(throttle.stats.queued).toBe(0);
  });
});
