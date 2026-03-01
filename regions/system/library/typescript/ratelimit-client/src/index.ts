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
    this.serverUrl = serverUrl.replace(/\/$/, '');
  }

  async check(key: string, cost: number): Promise<RateLimitStatus> {
    let response: Response;
    try {
      response = await fetch(`${this.serverUrl}/api/v1/ratelimit/${key}/check`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ cost }),
      });
    } catch (e) {
      const isAbort = e instanceof Error && e.name === 'AbortError';
      throw new RateLimitError(
        isAbort ? 'Request timed out' : 'Network error',
        'TIMEOUT',
      );
    }

    if (response.status === 429) {
      const body = await response.json().catch(() => ({}));
      throw new RateLimitError(
        'Rate limit exceeded',
        'LIMIT_EXCEEDED',
        body.retry_after_secs,
      );
    }
    if (response.status === 404) {
      throw new RateLimitError(`Key not found: ${key}`, 'KEY_NOT_FOUND');
    }
    if (!response.ok) {
      throw new RateLimitError(`Server error: ${response.status}`, 'SERVER_ERROR');
    }

    const body = await response.json();
    return {
      allowed: body.allowed,
      remaining: body.remaining,
      resetAt: new Date(body.reset_at),
      retryAfterSecs: body.retry_after_secs,
    };
  }

  async consume(key: string, cost: number): Promise<RateLimitResult> {
    let response: Response;
    try {
      response = await fetch(`${this.serverUrl}/api/v1/ratelimit/${key}/consume`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ cost }),
      });
    } catch (e) {
      const isAbort = e instanceof Error && e.name === 'AbortError';
      throw new RateLimitError(
        isAbort ? 'Request timed out' : 'Network error',
        'TIMEOUT',
      );
    }

    if (response.status === 429) {
      const body = await response.json().catch(() => ({}));
      throw new RateLimitError(
        'Rate limit exceeded',
        'LIMIT_EXCEEDED',
        body.retry_after_secs,
      );
    }
    if (response.status === 404) {
      throw new RateLimitError(`Key not found: ${key}`, 'KEY_NOT_FOUND');
    }
    if (!response.ok) {
      throw new RateLimitError(`Server error: ${response.status}`, 'SERVER_ERROR');
    }

    const body = await response.json();
    return {
      remaining: body.remaining,
      resetAt: new Date(body.reset_at),
    };
  }

  async getLimit(key: string): Promise<RateLimitPolicy> {
    let response: Response;
    try {
      response = await fetch(`${this.serverUrl}/api/v1/ratelimit/${key}/policy`);
    } catch (e) {
      const isAbort = e instanceof Error && e.name === 'AbortError';
      throw new RateLimitError(
        isAbort ? 'Request timed out' : 'Network error',
        'TIMEOUT',
      );
    }

    if (response.status === 404) {
      throw new RateLimitError(`Key not found: ${key}`, 'KEY_NOT_FOUND');
    }
    if (!response.ok) {
      throw new RateLimitError(`Server error: ${response.status}`, 'SERVER_ERROR');
    }

    const body = await response.json();
    return {
      key: body.key,
      limit: body.limit,
      windowSecs: body.window_secs,
      algorithm: body.algorithm,
    };
  }

  async close(): Promise<void> {
    // no-op (HTTP client has no persistent connection)
  }
}
