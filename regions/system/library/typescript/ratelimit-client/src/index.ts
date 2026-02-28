export interface RateLimitStatus {
  allowed: boolean;
  remaining: number;
  resetAt: Date;
  retryAfterSecs?: number;
}

export interface RateLimitResult {
  remaining: number;
  resetAt: Date;
}

export interface RateLimitPolicy {
  key: string;
  limit: number;
  windowSecs: number;
  algorithm: 'token_bucket' | 'sliding_window' | 'fixed_window';
}

export class RateLimitError extends Error {
  constructor(
    message: string,
    public readonly code: 'LIMIT_EXCEEDED' | 'KEY_NOT_FOUND' | 'SERVER_ERROR' | 'TIMEOUT',
    public readonly retryAfterSecs?: number,
  ) {
    super(message);
    this.name = 'RateLimitError';
  }
}

export interface RateLimitClient {
  check(key: string, cost: number): Promise<RateLimitStatus>;
  consume(key: string, cost: number): Promise<RateLimitResult>;
  getLimit(key: string): Promise<RateLimitPolicy>;
}

export class InMemoryRateLimitClient implements RateLimitClient {
  private counters = new Map<string, number>();
  private policies = new Map<string, RateLimitPolicy>();
  private defaultPolicy: RateLimitPolicy = {
    key: 'default',
    limit: 100,
    windowSecs: 3600,
    algorithm: 'token_bucket',
  };

  setPolicy(key: string, policy: RateLimitPolicy): void {
    this.policies.set(key, policy);
  }

  private getPolicy(key: string): RateLimitPolicy {
    return this.policies.get(key) ?? this.defaultPolicy;
  }

  async check(key: string, cost: number): Promise<RateLimitStatus> {
    const policy = this.getPolicy(key);
    const used = this.counters.get(key) ?? 0;
    const resetAt = new Date(Date.now() + policy.windowSecs * 1000);

    if (used + cost > policy.limit) {
      return {
        allowed: false,
        remaining: 0,
        resetAt,
        retryAfterSecs: policy.windowSecs,
      };
    }

    return {
      allowed: true,
      remaining: policy.limit - used - cost,
      resetAt,
    };
  }

  async consume(key: string, cost: number): Promise<RateLimitResult> {
    const policy = this.getPolicy(key);
    const used = this.counters.get(key) ?? 0;

    if (used + cost > policy.limit) {
      throw new RateLimitError(
        `Rate limit exceeded for key: ${key}`,
        'LIMIT_EXCEEDED',
        policy.windowSecs,
      );
    }

    this.counters.set(key, used + cost);
    const remaining = policy.limit - (used + cost);
    const resetAt = new Date(Date.now() + policy.windowSecs * 1000);

    return { remaining, resetAt };
  }

  async getLimit(key: string): Promise<RateLimitPolicy> {
    return this.getPolicy(key);
  }

  getUsedCount(key: string): number {
    return this.counters.get(key) ?? 0;
  }
}

export class GrpcRateLimitClient implements RateLimitClient {
  private readonly serverUrl: string;

  constructor(serverUrl: string) {
    this.serverUrl = serverUrl;
  }

  async check(_key: string, _cost: number): Promise<RateLimitStatus> {
    throw new RateLimitError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async consume(_key: string, _cost: number): Promise<RateLimitResult> {
    throw new RateLimitError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async getLimit(_key: string): Promise<RateLimitPolicy> {
    throw new RateLimitError('gRPC client not yet connected', 'SERVER_ERROR');
  }

  async close(): Promise<void> {
    // 接続クリーンアップ用プレースホルダー
  }
}
