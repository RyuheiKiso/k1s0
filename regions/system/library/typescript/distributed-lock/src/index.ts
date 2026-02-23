import { randomBytes } from 'crypto';

export interface LockGuard {
  key: string;
  token: string;
}

export class LockError extends Error {
  constructor(
    message: string,
    public code: string,
  ) {
    super(message);
    this.name = 'LockError';
  }
}

interface LockEntry {
  token: string;
  expiresAt: number;
}

export class InMemoryLock {
  private locks = new Map<string, LockEntry>();

  async acquire(key: string, ttlMs: number): Promise<LockGuard> {
    this.cleanup();
    const existing = this.locks.get(key);
    if (existing && existing.expiresAt > Date.now()) {
      throw new LockError(`lock already held: ${key}`, 'LOCK_HELD');
    }
    const token = randomBytes(16).toString('hex');
    this.locks.set(key, { token, expiresAt: Date.now() + ttlMs });
    return { key, token };
  }

  async release(guard: LockGuard): Promise<void> {
    const existing = this.locks.get(guard.key);
    if (!existing || existing.token !== guard.token) {
      throw new LockError(`lock not owned: ${guard.key}`, 'NOT_OWNER');
    }
    this.locks.delete(guard.key);
  }

  async isLocked(key: string): Promise<boolean> {
    this.cleanup();
    const existing = this.locks.get(key);
    return existing !== undefined && existing.expiresAt > Date.now();
  }

  private cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.locks) {
      if (entry.expiresAt <= now) {
        this.locks.delete(key);
      }
    }
  }
}
