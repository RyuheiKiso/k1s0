import { describe, it, expect } from 'vitest';
import {
  ResiliencyDecorator,
  ResiliencyError,
  withResiliency,
} from '../src/index.js';

describe('ResiliencyDecorator', () => {
  it('should execute successfully', async () => {
    const decorator = new ResiliencyDecorator({});
    const result = await decorator.execute(async () => 42);
    expect(result).toBe(42);
  });

  it('should retry on failure', async () => {
    const decorator = new ResiliencyDecorator({
      retry: {
        maxAttempts: 3,
        baseDelayMs: 10,
        maxDelayMs: 100,
      },
    });

    let counter = 0;
    const result = await decorator.execute(async () => {
      counter++;
      if (counter < 3) throw new Error('fail');
      return 99;
    });

    expect(result).toBe(99);
    expect(counter).toBe(3);
  });

  it('should throw MaxRetriesExceeded', async () => {
    const decorator = new ResiliencyDecorator({
      retry: {
        maxAttempts: 2,
        baseDelayMs: 1,
        maxDelayMs: 10,
      },
    });

    await expect(
      decorator.execute(async () => {
        throw new Error('always fail');
      }),
    ).rejects.toThrow(ResiliencyError);

    try {
      await decorator.execute(async () => {
        throw new Error('always fail');
      });
    } catch (err) {
      expect(err).toBeInstanceOf(ResiliencyError);
      expect((err as ResiliencyError).kind).toBe('retry_exceeded');
    }
  });

  it('should timeout', async () => {
    const decorator = new ResiliencyDecorator({
      timeoutMs: 50,
    });

    await expect(
      decorator.execute(
        () => new Promise((resolve) => setTimeout(() => resolve(42), 1000)),
      ),
    ).rejects.toThrow(ResiliencyError);
  });

  it('should open circuit breaker after failures', async () => {
    const decorator = new ResiliencyDecorator({
      circuitBreaker: {
        failureThreshold: 3,
        recoveryTimeoutMs: 60000,
        halfOpenMaxCalls: 1,
      },
    });

    // Trip the circuit
    for (let i = 0; i < 3; i++) {
      try {
        await decorator.execute(async () => {
          throw new Error('fail');
        });
      } catch {
        // expected
      }
    }

    // Next call should fail with circuit_open
    try {
      await decorator.execute(async () => 42);
      expect.unreachable('should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(ResiliencyError);
      expect((err as ResiliencyError).kind).toBe('circuit_open');
    }
  });

  it('should reject when bulkhead is full', async () => {
    const decorator = new ResiliencyDecorator({
      bulkhead: {
        maxConcurrentCalls: 1,
        maxWaitDurationMs: 50,
      },
    });

    // Occupy the single slot
    const longRunning = decorator.execute(
      () => new Promise((resolve) => setTimeout(() => resolve(1), 500)),
    );

    // Wait a tick for the first call to acquire the slot
    await new Promise((resolve) => setTimeout(resolve, 10));

    // This should fail with bulkhead_full
    try {
      await decorator.execute(async () => 2);
      expect.unreachable('should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(ResiliencyError);
      expect((err as ResiliencyError).kind).toBe('bulkhead_full');
    }

    await longRunning;
  });
});

describe('withResiliency', () => {
  it('should work as a convenience function', async () => {
    const result = await withResiliency({}, async () => 42);
    expect(result).toBe(42);
  });
});
