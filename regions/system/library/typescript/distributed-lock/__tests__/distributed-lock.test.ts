import { describe, it, expect } from 'vitest';
import { InMemoryLock, LockError } from '../src/index.js';

describe('InMemoryLock', () => {
  it('ロックを取得できる', async () => {
    const lock = new InMemoryLock();
    const guard = await lock.acquire('resource-1', 5000);
    expect(guard.key).toBe('resource-1');
    expect(guard.token).toBeTruthy();
  });

  it('同じキーの二重ロックでLockErrorを投げる', async () => {
    const lock = new InMemoryLock();
    await lock.acquire('resource-1', 5000);
    await expect(lock.acquire('resource-1', 5000)).rejects.toThrow(LockError);
  });

  it('リリース後に再取得できる', async () => {
    const lock = new InMemoryLock();
    const guard = await lock.acquire('resource-1', 5000);
    await lock.release(guard);
    const guard2 = await lock.acquire('resource-1', 5000);
    expect(guard2.key).toBe('resource-1');
  });

  it('isLockedがロック状態を正しく返す', async () => {
    const lock = new InMemoryLock();
    expect(await lock.isLocked('key')).toBe(false);
    const guard = await lock.acquire('key', 5000);
    expect(await lock.isLocked('key')).toBe(true);
    await lock.release(guard);
    expect(await lock.isLocked('key')).toBe(false);
  });

  it('異なるトークンでリリースするとLockErrorを投げる', async () => {
    const lock = new InMemoryLock();
    await lock.acquire('key', 5000);
    await expect(lock.release({ key: 'key', token: 'wrong-token' })).rejects.toThrow(LockError);
  });
});
