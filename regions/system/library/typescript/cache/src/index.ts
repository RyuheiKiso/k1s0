export class CacheError extends Error {
  constructor(
    message: string,
    public readonly code: string,
  ) {
    super(message);
    this.name = 'CacheError';
  }
}

export interface CacheClient {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, ttlMs?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
  exists(key: string): Promise<boolean>;
  setNX(key: string, value: string, ttlMs: number): Promise<boolean>;
  // 指定キーの有効期限をミリ秒単位で更新する
  expire(key: string, ttlMs: number): Promise<boolean>;
}

interface RedisLike {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, mode?: string, duration?: number, nx?: string): Promise<"OK" | null>;
  del(key: string): Promise<number>;
  exists(key: string): Promise<number>;
  // PEXPIREコマンドでキーの有効期限をミリ秒単位で設定する
  pexpire(key: string, ttlMs: number): Promise<number>;
}

interface Entry {
  value: string;
  expiresAt: number | null;
}

export class InMemoryCacheClient implements CacheClient {
  private entries = new Map<string, Entry>();

  async get(key: string): Promise<string | null> {
    const entry = this.entries.get(key);
    if (!entry) return null;
    if (this.isExpired(entry)) {
      this.entries.delete(key);
      return null;
    }
    return entry.value;
  }

  async set(key: string, value: string, ttlMs?: number): Promise<void> {
    this.entries.set(key, {
      value,
      expiresAt: ttlMs != null ? Date.now() + ttlMs : null,
    });
  }

  async delete(key: string): Promise<boolean> {
    return this.entries.delete(key);
  }

  async exists(key: string): Promise<boolean> {
    const entry = this.entries.get(key);
    if (!entry) return false;
    if (this.isExpired(entry)) {
      this.entries.delete(key);
      return false;
    }
    return true;
  }

  async setNX(key: string, value: string, ttlMs: number): Promise<boolean> {
    const entry = this.entries.get(key);
    if (entry && !this.isExpired(entry)) return false;
    this.entries.set(key, {
      value,
      expiresAt: Date.now() + ttlMs,
    });
    return true;
  }

  // 指定キーの有効期限を更新する。キーが存在しないまたは期限切れの場合はfalseを返す
  async expire(key: string, ttlMs: number): Promise<boolean> {
    const entry = this.entries.get(key);
    if (!entry) return false;
    if (this.isExpired(entry)) {
      this.entries.delete(key);
      return false;
    }
    this.entries.set(key, {
      value: entry.value,
      expiresAt: Date.now() + ttlMs,
    });
    return true;
  }

  private isExpired(entry: Entry): boolean {
    return entry.expiresAt !== null && entry.expiresAt <= Date.now();
  }
}

export class RedisCacheClient implements CacheClient {
  private constructor(
    private readonly redis: RedisLike,
    private readonly keyPrefix = '',
  ) {}

  static async fromUrl(url: string, keyPrefix = ''): Promise<RedisCacheClient> {
    // ioredis の動的インポートで Redis コンストラクタを取得し、RedisLike として扱う
    const { default: Redis } = await import('ioredis');
    const redis = new (Redis as unknown as new (url: string) => RedisLike)(url);
    return new RedisCacheClient(redis, keyPrefix);
  }

  static fromClient(redis: RedisLike, keyPrefix = ''): RedisCacheClient {
    return new RedisCacheClient(redis, keyPrefix);
  }

  async get(key: string): Promise<string | null> {
    return this.redis.get(this.prefixedKey(key));
  }

  async set(key: string, value: string, ttlMs?: number): Promise<void> {
    const redisKey = this.prefixedKey(key);
    if (ttlMs != null) {
      await this.redis.set(redisKey, value, 'PX', ttlMs);
      return;
    }
    await this.redis.set(redisKey, value);
  }

  async delete(key: string): Promise<boolean> {
    return (await this.redis.del(this.prefixedKey(key))) > 0;
  }

  async exists(key: string): Promise<boolean> {
    return (await this.redis.exists(this.prefixedKey(key))) > 0;
  }

  async setNX(key: string, value: string, ttlMs: number): Promise<boolean> {
    const result = await this.redis.set(this.prefixedKey(key), value, 'PX', ttlMs, 'NX');
    return result === 'OK';
  }

  // PEXPIREコマンドでキーの有効期限をミリ秒単位で設定する
  async expire(key: string, ttlMs: number): Promise<boolean> {
    const result = await this.redis.pexpire(this.prefixedKey(key), ttlMs);
    return result > 0;
  }

  private prefixedKey(key: string): string {
    return this.keyPrefix ? `${this.keyPrefix}:${key}` : key;
  }
}
