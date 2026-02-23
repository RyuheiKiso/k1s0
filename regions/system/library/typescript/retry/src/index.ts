export class RetryError extends Error {
  constructor(
    public readonly attempts: number,
    public readonly lastError: Error,
  ) {
    super(`exhausted ${attempts} retries: ${lastError.message}`);
    this.name = 'RetryError';
  }
}

export interface RetryConfig {
  maxAttempts: number;
  initialDelayMs: number;
  maxDelayMs: number;
  multiplier: number;
  jitter: boolean;
}

export const defaultRetryConfig: RetryConfig = {
  maxAttempts: 3,
  initialDelayMs: 100,
  maxDelayMs: 30_000,
  multiplier: 2.0,
  jitter: true,
};

export function computeDelay(config: RetryConfig, attempt: number): number {
  const base = config.initialDelayMs * Math.pow(config.multiplier, attempt);
  const capped = Math.min(base, config.maxDelayMs);
  if (config.jitter) {
    const jitterRange = capped * 0.1;
    return capped - jitterRange + Math.random() * jitterRange * 2;
  }
  return capped;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function withRetry<T>(
  config: RetryConfig,
  operation: () => Promise<T>,
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt < config.maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));
      if (attempt + 1 < config.maxAttempts) {
        const delay = computeDelay(config, attempt);
        await sleep(delay);
      }
    }
  }
  throw new RetryError(config.maxAttempts, lastError!);
}

export type CircuitBreakerState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeoutMs: number;
}

export const defaultCircuitBreakerConfig: CircuitBreakerConfig = {
  failureThreshold: 5,
  successThreshold: 2,
  timeoutMs: 30_000,
};

export class CircuitBreaker {
  private state: CircuitBreakerState = 'closed';
  private failureCount = 0;
  private successCount = 0;
  private openedAt = 0;
  private readonly config: CircuitBreakerConfig;

  constructor(config: Partial<CircuitBreakerConfig> = {}) {
    this.config = { ...defaultCircuitBreakerConfig, ...config };
  }

  getState(): CircuitBreakerState {
    this.checkTimeout();
    return this.state;
  }

  isOpen(): boolean {
    this.checkTimeout();
    return this.state === 'open';
  }

  recordSuccess(): void {
    this.checkTimeout();
    if (this.state === 'half-open') {
      this.successCount++;
      if (this.successCount >= this.config.successThreshold) {
        this.state = 'closed';
        this.failureCount = 0;
        this.successCount = 0;
        this.openedAt = 0;
      }
    } else if (this.state === 'closed') {
      this.failureCount = 0;
    }
  }

  recordFailure(): void {
    this.checkTimeout();
    this.failureCount++;
    if (this.failureCount >= this.config.failureThreshold) {
      this.state = 'open';
      this.openedAt = Date.now();
      this.failureCount = 0;
    }
  }

  private checkTimeout(): void {
    if (this.state === 'open' && this.openedAt > 0) {
      if (Date.now() - this.openedAt >= this.config.timeoutMs) {
        this.state = 'half-open';
        this.successCount = 0;
      }
    }
  }
}
