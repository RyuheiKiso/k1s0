import {
  CircuitBreaker,
  CircuitBreakerError,
} from '@k1s0/circuit-breaker';
import { Bulkhead, BulkheadFullError } from '@k1s0/bulkhead';

export interface RetryConfig {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  jitter?: boolean;
}

export interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeoutMs: number;
  halfOpenMaxCalls?: number;
}

export interface BulkheadConfig {
  maxConcurrentCalls: number;
  maxWaitDurationMs: number;
}

export interface ResiliencyPolicy {
  retry?: RetryConfig;
  circuitBreaker?: CircuitBreakerConfig;
  bulkhead?: BulkheadConfig;
  timeoutMs?: number;
}

export type ResiliencyErrorKind =
  | 'retry_exceeded'
  | 'circuit_open'
  | 'bulkhead_full'
  | 'timeout';

export class ResiliencyError extends Error {
  constructor(
    message: string,
    public readonly kind: ResiliencyErrorKind,
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = 'ResiliencyError';
  }
}

export class ResiliencyDecorator {
  private readonly policy: ResiliencyPolicy;
  private readonly bulkhead?: Bulkhead;
  private readonly cb?: CircuitBreaker;

  constructor(policy: ResiliencyPolicy) {
    this.policy = policy;

    if (policy.bulkhead) {
      this.bulkhead = new Bulkhead({
        maxConcurrentCalls: policy.bulkhead.maxConcurrentCalls,
        maxWaitDurationMs: policy.bulkhead.maxWaitDurationMs,
      });
    }

    if (policy.circuitBreaker) {
      this.cb = new CircuitBreaker({
        failureThreshold: policy.circuitBreaker.failureThreshold,
        successThreshold: policy.circuitBreaker.halfOpenMaxCalls ?? 1,
        timeoutMs: policy.circuitBreaker.recoveryTimeoutMs,
      });
    }
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    this.checkCircuitBreaker();

    if (this.bulkhead) {
      try {
        await this.bulkhead.acquire();
      } catch (err) {
        if (err instanceof BulkheadFullError) {
          throw new ResiliencyError(
            `bulkhead full, max concurrent: ${this.policy.bulkhead!.maxConcurrentCalls}`,
            'bulkhead_full',
          );
        }
        throw err;
      }
    }

    try {
      return await this.executeWithRetry(fn);
    } finally {
      if (this.bulkhead) {
        this.bulkhead.release();
      }
    }
  }

  private async executeWithRetry<T>(fn: () => Promise<T>): Promise<T> {
    const maxAttempts = this.policy.retry?.maxAttempts ?? 1;
    let lastError: Error | undefined;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        const result = await this.executeWithTimeout(fn);
        this.recordSuccess();
        return result;
      } catch (err) {
        this.recordFailure();
        lastError = err instanceof Error ? err : new Error(String(err));

        try {
          this.checkCircuitBreaker();
        } catch (cbErr) {
          throw cbErr;
        }

        if (attempt + 1 < maxAttempts && this.policy.retry) {
          const delay = calculateBackoff(
            attempt,
            this.policy.retry.baseDelayMs,
            this.policy.retry.maxDelayMs,
          );
          await sleep(delay);
        }
      }
    }

    throw new ResiliencyError(
      `max retries exceeded after ${maxAttempts} attempts`,
      'retry_exceeded',
      lastError,
    );
  }

  private async executeWithTimeout<T>(fn: () => Promise<T>): Promise<T> {
    if (!this.policy.timeoutMs) {
      return fn();
    }

    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(
          new ResiliencyError(
            `timed out after ${this.policy.timeoutMs}ms`,
            'timeout',
          ),
        );
      }, this.policy.timeoutMs!);

      fn()
        .then((result) => {
          clearTimeout(timer);
          resolve(result);
        })
        .catch((err) => {
          clearTimeout(timer);
          reject(err);
        });
    });
  }

  private checkCircuitBreaker(): void {
    if (!this.cb) return;

    if (this.cb.isOpen()) {
      throw new ResiliencyError(
        'circuit breaker open',
        'circuit_open',
      );
    }
  }

  private recordSuccess(): void {
    if (this.cb) {
      this.cb.recordSuccess();
    }
  }

  private recordFailure(): void {
    if (this.cb) {
      this.cb.recordFailure();
    }
  }
}

export async function withResiliency<T>(
  policy: ResiliencyPolicy,
  fn: () => Promise<T>,
): Promise<T> {
  const decorator = new ResiliencyDecorator(policy);
  return decorator.execute(fn);
}

function calculateBackoff(
  attempt: number,
  baseDelayMs: number,
  maxDelayMs: number,
): number {
  const delay = baseDelayMs * Math.pow(2, attempt);
  return Math.min(delay, maxDelayMs);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
