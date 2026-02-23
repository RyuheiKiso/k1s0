export type CircuitState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeoutMs: number;
}

export class CircuitBreakerError extends Error {
  constructor() {
    super('Circuit breaker is open');
    this.name = 'CircuitBreakerError';
  }
}

export class CircuitBreaker {
  private _state: CircuitState = 'closed';
  private failureCount = 0;
  private successCount = 0;
  private openedAt = 0;
  private readonly config: CircuitBreakerConfig;

  constructor(config: CircuitBreakerConfig) {
    this.config = config;
  }

  get state(): CircuitState {
    this.checkTimeout();
    return this._state;
  }

  isOpen(): boolean {
    this.checkTimeout();
    return this._state === 'open';
  }

  recordSuccess(): void {
    this.checkTimeout();
    if (this._state === 'half-open') {
      this.successCount++;
      if (this.successCount >= this.config.successThreshold) {
        this._state = 'closed';
        this.failureCount = 0;
        this.successCount = 0;
        this.openedAt = 0;
      }
    } else if (this._state === 'closed') {
      this.failureCount = 0;
    }
  }

  recordFailure(): void {
    this.checkTimeout();
    this.failureCount++;
    if (this.failureCount >= this.config.failureThreshold) {
      this._state = 'open';
      this.openedAt = Date.now();
      this.failureCount = 0;
    }
  }

  async call<T>(fn: () => Promise<T>): Promise<T> {
    if (this.isOpen()) {
      throw new CircuitBreakerError();
    }
    try {
      const result = await fn();
      this.recordSuccess();
      return result;
    } catch (err) {
      this.recordFailure();
      throw err;
    }
  }

  private checkTimeout(): void {
    if (this._state === 'open' && this.openedAt > 0) {
      if (Date.now() - this.openedAt >= this.config.timeoutMs) {
        this._state = 'half-open';
        this.successCount = 0;
      }
    }
  }
}
