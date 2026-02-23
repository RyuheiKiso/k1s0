export type QuotaPeriod = 'hourly' | 'daily' | 'monthly' | { customMs: number };

export interface QuotaStatus {
  allowed: boolean;
  remaining: number;
  limit: number;
  resetAt: Date;
}

export interface QuotaUsage {
  quotaId: string;
  used: number;
  limit: number;
  period: QuotaPeriod;
  resetAt: Date;
}

export interface QuotaPolicy {
  quotaId: string;
  limit: number;
  period: QuotaPeriod;
  resetStrategy: 'sliding' | 'fixed';
}

export interface QuotaClientConfig {
  serverUrl: string;
  timeoutMs?: number;
  policyCacheTtlMs?: number;
}

export class QuotaExceededError extends Error {
  constructor(
    public readonly quotaId: string,
    public readonly remaining: number,
  ) {
    super(`Quota exceeded: ${quotaId}, remaining=${remaining}`);
    this.name = 'QuotaExceededError';
  }
}

export interface QuotaClient {
  check(quotaId: string, amount: number): Promise<QuotaStatus>;
  increment(quotaId: string, amount: number): Promise<QuotaUsage>;
  getUsage(quotaId: string): Promise<QuotaUsage>;
  getPolicy(quotaId: string): Promise<QuotaPolicy>;
}

export class InMemoryQuotaClient implements QuotaClient {
  private usages = new Map<string, QuotaUsage>();
  private policies = new Map<string, QuotaPolicy>();

  setPolicy(quotaId: string, policy: QuotaPolicy): void {
    this.policies.set(quotaId, policy);
  }

  private getOrCreateUsage(quotaId: string): QuotaUsage {
    let usage = this.usages.get(quotaId);
    if (!usage) {
      const policy = this.policies.get(quotaId);
      usage = {
        quotaId,
        used: 0,
        limit: policy?.limit ?? 1000,
        period: policy?.period ?? 'daily',
        resetAt: new Date(Date.now() + 86400000),
      };
      this.usages.set(quotaId, usage);
    }
    return usage;
  }

  async check(quotaId: string, amount: number): Promise<QuotaStatus> {
    const usage = this.getOrCreateUsage(quotaId);
    const remaining = usage.limit - usage.used;
    return {
      allowed: amount <= remaining,
      remaining,
      limit: usage.limit,
      resetAt: usage.resetAt,
    };
  }

  async increment(quotaId: string, amount: number): Promise<QuotaUsage> {
    const usage = this.getOrCreateUsage(quotaId);
    usage.used += amount;
    return { ...usage };
  }

  async getUsage(quotaId: string): Promise<QuotaUsage> {
    const usage = this.getOrCreateUsage(quotaId);
    return { ...usage };
  }

  async getPolicy(quotaId: string): Promise<QuotaPolicy> {
    const policy = this.policies.get(quotaId);
    if (policy) return { ...policy };
    return {
      quotaId,
      limit: 1000,
      period: 'daily',
      resetStrategy: 'fixed',
    };
  }
}

interface CacheEntry {
  policy: QuotaPolicy;
  expiresAt: number;
}

export class CachedQuotaClient implements QuotaClient {
  private cache = new Map<string, CacheEntry>();

  constructor(
    private readonly inner: QuotaClient,
    private readonly policyTtlMs: number,
  ) {}

  check(quotaId: string, amount: number): Promise<QuotaStatus> {
    return this.inner.check(quotaId, amount);
  }

  increment(quotaId: string, amount: number): Promise<QuotaUsage> {
    return this.inner.increment(quotaId, amount);
  }

  getUsage(quotaId: string): Promise<QuotaUsage> {
    return this.inner.getUsage(quotaId);
  }

  async getPolicy(quotaId: string): Promise<QuotaPolicy> {
    const cached = this.cache.get(quotaId);
    if (cached && Date.now() < cached.expiresAt) {
      return cached.policy;
    }
    const policy = await this.inner.getPolicy(quotaId);
    this.cache.set(quotaId, {
      policy,
      expiresAt: Date.now() + this.policyTtlMs,
    });
    return policy;
  }
}
