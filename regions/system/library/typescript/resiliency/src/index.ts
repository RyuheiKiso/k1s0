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

type CircuitState = 'closed' | 'open' | 'half_open';

export class ResiliencyDecorator {
  private readonly policy: ResiliencyPolicy;
  private bulkheadCurrent = 0;
  private bulkheadWaiters: Array<{
    resolve: () => void;
    timer: ReturnType<typeof setTimeout>;
  }> = [];
  private cbState: CircuitState = 'closed';
  private cbFailureCount = 0;
  private cbSuccessCount = 0;
  private cbLastFailureTime = 0;

  constructor(policy: ResiliencyPolicy) {
    this.policy = policy;
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    this.checkCircuitBreaker();

    if (this.policy.bulkhead) {
      await this.acquireBulkhead();
    }

    try {
      return await this.executeWithRetry(fn);
    } finally {
      if (this.policy.bulkhead) {
        this.releaseBulkhead();
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
    if (!this.policy.circuitBreaker) return;

    const cfg = this.policy.circuitBreaker;

    switch (this.cbState) {
      case 'closed':
        return;
      case 'open': {
        const elapsed = Date.now() - this.cbLastFailureTime;
        if (elapsed >= cfg.recoveryTimeoutMs) {
          this.cbState = 'half_open';
          this.cbSuccessCount = 0;
          return;
        }
        throw new ResiliencyError(
          `circuit breaker open, remaining: ${cfg.recoveryTimeoutMs - elapsed}ms`,
          'circuit_open',
        );
      }
      case 'half_open':
        return;
    }
  }

  private recordSuccess(): void {
    if (!this.policy.circuitBreaker) return;

    if (this.cbState === 'half_open') {
      this.cbSuccessCount++;
      const maxCalls = this.policy.circuitBreaker.halfOpenMaxCalls ?? 1;
      if (this.cbSuccessCount >= maxCalls) {
        this.cbState = 'closed';
        this.cbFailureCount = 0;
      }
    } else if (this.cbState === 'closed') {
      this.cbFailureCount = 0;
    }
  }

  private recordFailure(): void {
    if (!this.policy.circuitBreaker) return;

    this.cbFailureCount++;
    if (this.cbFailureCount >= this.policy.circuitBreaker.failureThreshold) {
      this.cbState = 'open';
      this.cbLastFailureTime = Date.now();
    }
  }

  private acquireBulkhead(): Promise<void> {
    const cfg = this.policy.bulkhead!;
    if (this.bulkheadCurrent < cfg.maxConcurrentCalls) {
      this.bulkheadCurrent++;
      return Promise.resolve();
    }

    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        const idx = this.bulkheadWaiters.findIndex((w) => w.resolve === resolve);
        if (idx >= 0) this.bulkheadWaiters.splice(idx, 1);
        reject(
          new ResiliencyError(
            `bulkhead full, max concurrent: ${cfg.maxConcurrentCalls}`,
            'bulkhead_full',
          ),
        );
      }, cfg.maxWaitDurationMs);

      this.bulkheadWaiters.push({ resolve, timer });
    });
  }

  private releaseBulkhead(): void {
    if (this.bulkheadWaiters.length > 0) {
      const waiter = this.bulkheadWaiters.shift()!;
      clearTimeout(waiter.timer);
      waiter.resolve();
    } else {
      this.bulkheadCurrent--;
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
