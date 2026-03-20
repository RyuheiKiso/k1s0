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

// サーキットブレーカーは正規モジュールから再エクスポートする（重複実装を排除）
export {
  CircuitBreaker,
  CircuitBreakerError,
  type CircuitBreakerConfig,
  type CircuitState,
} from '@k1s0/circuit-breaker';
