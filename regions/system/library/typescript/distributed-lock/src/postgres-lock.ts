import { randomBytes } from 'crypto';
import type { Pool } from 'pg';
import { LockError, type LockGuard } from './index.js';

export class PostgresDistributedLock {
  private readonly keyPrefix: string;

  constructor(
    private readonly pool: Pool,
    keyPrefix = 'lock',
  ) {
    this.keyPrefix = keyPrefix;
  }

  private fullKey(key: string): string {
    return `${this.keyPrefix}:${key}`;
  }

  async acquire(key: string, _ttlMs: number): Promise<LockGuard> {
    const full = this.fullKey(key);
    const token = randomBytes(16).toString('hex');

    const result = await this.pool.query<{ pg_try_advisory_lock: boolean }>(
      'SELECT pg_try_advisory_lock(hashtext($1))',
      [full],
    );

    if (!result.rows[0].pg_try_advisory_lock) {
      throw new LockError(`lock already held: ${key}`, 'LOCK_HELD');
    }

    return { key, token };
  }

  async release(guard: LockGuard): Promise<void> {
    const full = this.fullKey(guard.key);

    const result = await this.pool.query<{ pg_advisory_unlock: boolean }>(
      'SELECT pg_advisory_unlock(hashtext($1))',
      [full],
    );

    if (!result.rows[0].pg_advisory_unlock) {
      throw new LockError(`lock not owned: ${guard.key}`, 'NOT_OWNER');
    }
  }

  async isLocked(key: string): Promise<boolean> {
    const full = this.fullKey(key);

    const result = await this.pool.query<{ exists: boolean }>(
      "SELECT EXISTS(SELECT 1 FROM pg_locks WHERE locktype = 'advisory' AND classid = hashtext($1)::int) AS exists",
      [full],
    );

    return result.rows[0].exists;
  }
}
