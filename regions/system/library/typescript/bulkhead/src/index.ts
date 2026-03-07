export interface BulkheadConfig {
  maxConcurrentCalls: number;
  maxWaitDurationMs: number;
}

export class BulkheadFullError extends Error {
  constructor() {
    super('Bulkhead is full');
    this.name = 'BulkheadFullError';
  }
}

export class Bulkhead {
  private current = 0;
  private readonly waiters: Array<{ resolve: () => void; timer: ReturnType<typeof setTimeout> }> = [];
  private readonly config: BulkheadConfig;

  constructor(config: BulkheadConfig) {
    this.config = config;
  }

  async acquire(): Promise<void> {
    if (this.current < this.config.maxConcurrentCalls) {
      this.current++;
      return;
    }

    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        const idx = this.waiters.findIndex((w) => w.resolve === resolve);
        if (idx !== -1) this.waiters.splice(idx, 1);
        reject(new BulkheadFullError());
      }, this.config.maxWaitDurationMs);

      this.waiters.push({ resolve, timer });
    });
  }

  release(): void {
    if (this.waiters.length > 0) {
      const waiter = this.waiters.shift()!;
      clearTimeout(waiter.timer);
      waiter.resolve();
    } else {
      this.current--;
    }
  }

  async call<T>(fn: () => Promise<T>): Promise<T> {
    await this.acquire();
    try {
      return await fn();
    } finally {
      this.release();
    }
  }
}
