/**
 * リクエストスロットル
 *
 * トークンバケット + 同時接続制限 + キュー上限によるリクエストレート制御。
 */
export interface ThrottleConfig {
  /** 1秒あたりの最大リクエスト数 */
  maxRequestsPerSecond: number;
  /** 同時接続数の上限 */
  maxConcurrent: number;
  /** キュー上限（超過時は即座にリジェクト） */
  maxQueueSize: number;
}

export const DEFAULT_THROTTLE_CONFIG: ThrottleConfig = {
  maxRequestsPerSecond: 10,
  maxConcurrent: 5,
  maxQueueSize: 50,
};

export class RequestThrottle {
  private queue: Array<() => void> = [];
  private activeCount = 0;
  private tokens: number;
  private readonly config: ThrottleConfig;
  private readonly timer: ReturnType<typeof setInterval>;
  private _allowed = 0;
  private _rejected = 0;

  constructor(config: Partial<ThrottleConfig> = {}) {
    this.config = { ...DEFAULT_THROTTLE_CONFIG, ...config };
    this.tokens = this.config.maxRequestsPerSecond;
    this.timer = setInterval(() => {
      this.tokens = this.config.maxRequestsPerSecond;
      this.processQueue();
    }, 1000);
  }

  async acquire(): Promise<void> {
    if (this.tokens > 0 && this.activeCount < this.config.maxConcurrent) {
      this.tokens--;
      this.activeCount++;
      this._allowed++;
      return;
    }
    if (this.queue.length >= this.config.maxQueueSize) {
      this._rejected++;
      throw new Error('Request throttle queue full');
    }
    return new Promise<void>((resolve) => {
      this.queue.push(() => {
        this.activeCount++;
        this._allowed++;
        resolve();
      });
    });
  }

  release(): void {
    this.activeCount--;
    this.processQueue();
  }

  get stats() {
    return {
      allowed: this._allowed,
      rejected: this._rejected,
      queued: this.queue.length,
      active: this.activeCount,
      availableTokens: this.tokens,
    };
  }

  dispose(): void {
    clearInterval(this.timer);
    this.queue = [];
  }

  private processQueue(): void {
    while (
      this.queue.length > 0 &&
      this.tokens > 0 &&
      this.activeCount < this.config.maxConcurrent
    ) {
      this.tokens--;
      const next = this.queue.shift();
      next?.();
    }
  }
}
