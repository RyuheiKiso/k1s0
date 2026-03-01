import { randomBytes } from 'crypto';
import type { Redis } from 'ioredis';
import { LockError, type LockGuard } from './index.js';

const RELEASE_SCRIPT = `
if redis.call("get", KEYS[1]) == ARGV[1] then
  return redis.call("del", KEYS[1])
else
  return 0
end
`;

const EXTEND_SCRIPT = `
if redis.call("get", KEYS[1]) == ARGV[1] then
  return redis.call("pexpire", KEYS[1], ARGV[2])
else
  return 0
end
`;

export class RedisDistributedLock {
  private readonly keyPrefix: string;

  constructor(
    private readonly redis: Redis,
    keyPrefix = 'lock',
  ) {
    this.keyPrefix = keyPrefix;
  }

  private lockKey(key: string): string {
    return `${this.keyPrefix}:${key}`;
  }

  async acquire(key: string, ttlMs: number): Promise<LockGuard> {
    const fullKey = this.lockKey(key);
    const token = randomBytes(16).toString('hex');

    const result = await this.redis.set(fullKey, token, 'PX', ttlMs, 'NX');

    if (result === null) {
      throw new LockError(`lock already held: ${key}`, 'LOCK_HELD');
    }

    return { key, token };
  }

  async release(guard: LockGuard): Promise<void> {
    const fullKey = this.lockKey(guard.key);
    const result = await this.redis.eval(RELEASE_SCRIPT, 1, fullKey, guard.token) as number;

    if (result !== 1) {
      throw new LockError(`lock not owned: ${guard.key}`, 'NOT_OWNER');
    }
  }

  async extend(guard: LockGuard, ttlMs: number): Promise<void> {
    const fullKey = this.lockKey(guard.key);
    const result = await this.redis.eval(EXTEND_SCRIPT, 1, fullKey, guard.token, String(ttlMs)) as number;

    if (result !== 1) {
      throw new LockError(`lock not owned: ${guard.key}`, 'NOT_OWNER');
    }
  }

  async isLocked(key: string): Promise<boolean> {
    const fullKey = this.lockKey(key);
    const exists = await this.redis.exists(fullKey);
    return exists === 1;
  }
}
