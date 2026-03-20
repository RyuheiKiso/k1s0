export type CircuitState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeoutMs: number;
}

/** メトリクスのスナップショット型 */
export interface CircuitBreakerMetrics {
  /** 成功回数の累計 */
  successCount: number;
  /** 失敗回数の累計 */
  failureCount: number;
  /** 現在の状態文字列（"Closed" / "Open" / "HalfOpen"） */
  state: string;
}

export class CircuitBreakerError extends Error {
  constructor() {
    super('Circuit breaker is open');
    this.name = 'CircuitBreakerError';
  }
}

/** 状態を表示用文字列に変換する */
function stateToString(s: CircuitState): string {
  switch (s) {
    case 'open':
      return 'Open';
    case 'half-open':
      return 'HalfOpen';
    default:
      return 'Closed';
  }
}

export class CircuitBreaker {
  private _state: CircuitState = 'closed';
  private failureCount = 0;
  private successCount = 0;
  private openedAt = 0;
  private readonly config: CircuitBreakerConfig;

  // メトリクス累計カウンタ
  private _metricsSuccessCount = 0;
  private _metricsFailureCount = 0;

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
    // メトリクスに成功を記録する
    this._metricsSuccessCount++;
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
      // Closed 状態では成功時に失敗カウントをリセットする
      this.failureCount = 0;
    }
  }

  recordFailure(): void {
    // メトリクスに失敗を記録する
    this._metricsFailureCount++;
    this.checkTimeout();
    if (this._state === 'half-open') {
      // HalfOpen 状態での失敗は即座に Open へ再遷移する
      this._state = 'open';
      this.openedAt = Date.now();
      this.failureCount = 0;
      this.successCount = 0;
      return;
    }
    this.failureCount++;
    if (this.failureCount >= this.config.failureThreshold) {
      this._state = 'open';
      this.openedAt = Date.now();
      this.failureCount = 0;
    }
  }

  /** 現在のメトリクススナップショットを返す */
  metrics(): CircuitBreakerMetrics {
    return {
      successCount: this._metricsSuccessCount,
      failureCount: this._metricsFailureCount,
      state: stateToString(this._state),
    };
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
