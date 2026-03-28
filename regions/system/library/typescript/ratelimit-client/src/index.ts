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

/**
 * key を "scope:identifier" 形式から { scope, identifier } に分割する。
 * ":" が含まれない場合は scope="default"、identifier=key とする。
 */
function splitKey(key: string): { scope: string; identifier: string } {
  const idx = key.indexOf(':');
  if (idx >= 0) {
    return { scope: key.slice(0, idx), identifier: key.slice(idx + 1) };
  }
  return { scope: 'default', identifier: key };
}

/**
 * C-02/L-16 監査対応: GrpcRateLimitClient → HttpRateLimitClient にリネーム。
 * API パスをサーバー実装（POST /api/v1/ratelimit/check 等）に合わせて統一。
 */
export class HttpRateLimitClient implements RateLimitClient {
  private readonly serverUrl: string;

  constructor(serverUrl: string) {
    this.serverUrl = serverUrl.replace(/\/$/, '');
  }

  // C-02 監査対応: POST /api/v1/ratelimit/check（key をパスではなくボディに含める）
  async check(key: string, cost: number): Promise<RateLimitStatus> {
    const { scope, identifier } = splitKey(key);
    let response: Response;
    try {
      response = await fetch(`${this.serverUrl}/api/v1/ratelimit/check`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scope, identifier, window: cost > 1 ? `${cost}s` : undefined }),
      });
    } catch (e: unknown) {
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

  // C-02 監査対応: consume は check で代用（サーバーに consume エンドポイントはない）
  async consume(key: string, cost: number): Promise<RateLimitResult> {
    const status = await this.check(key, cost);
    return {
      remaining: status.remaining,
      resetAt: status.resetAt,
    };
  }

  // C-02 監査対応: GET /api/v1/ratelimit/usage
  async getLimit(key: string): Promise<RateLimitPolicy> {
    let response: Response;
    try {
      response = await fetch(`${this.serverUrl}/api/v1/ratelimit/usage`);
    } catch (e: unknown) {
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
      key: body.key || key,
      limit: body.limit,
      windowSecs: body.window_secs,
      algorithm: body.algorithm,
    };
  }

  async close(): Promise<void> {
    // no-op（HTTP クライアントには永続接続がない）
  }
}

/**
 * 後方互換性のための型エイリアス（L-16 監査対応: 旧名称からの移行期間用）。
 * @deprecated HttpRateLimitClient を使用してください。
 */
export const GrpcRateLimitClient = HttpRateLimitClient;
