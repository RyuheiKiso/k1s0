import { describe, it, expect, vi } from 'vitest';
import { RedisDistributedLock } from '../src/redis-lock.js';
import { LockError } from '../src/index.js';
import type Redis from 'ioredis';

function makeMockRedis(overrides: Partial<{
  set: (...args: unknown[]) => Promise<string | null>;
  eval: (...args: unknown[]) => Promise<number>;
  exists: (...args: unknown[]) => Promise<number>;
}>): Redis {
  return {
    set: vi.fn().mockImplementation(overrides.set ?? (() => Promise.resolve('OK'))),
    eval: vi.fn().mockImplementation(overrides.eval ?? (() => Promise.resolve(1))),
    exists: vi.fn().mockImplementation(overrides.exists ?? (() => Promise.resolve(0))),
  } as unknown as Redis;
}

describe('RedisDistributedLock', () => {
  it('acquire()でロックを取得できる', async () => {
    const redis = makeMockRedis({ set: () => Promise.resolve('OK') });
    const lock = new RedisDistributedLock(redis);
    const guard = await lock.acquire('resource', 5000);
    expect(guard.key).toBe('resource');
    expect(guard.token).toBeTruthy();
  });

  it('acquire()でSET NX PXコマンドが送られる', async () => {
    const redis = makeMockRedis({ set: () => Promise.resolve('OK') });
    const lock = new RedisDistributedLock(redis);
    await lock.acquire('resource', 5000);
    const setFn = redis.set as ReturnType<typeof vi.fn>;
    expect(setFn.mock.calls[0]).toEqual(
      expect.arrayContaining(['lock:resource', expect.any(String), 'NX', 'PX', 5000]),
    );
  });

  it('acquire()でロック取得失敗時にLockErrorを投げる', async () => {
    const redis = makeMockRedis({ set: () => Promise.resolve(null) });
    const lock = new RedisDistributedLock(redis);
    await expect(lock.acquire('resource', 5000)).rejects.toThrow(LockError);
  });

  it('release()でLuaスクリプトによるアトミック削除が実行される', async () => {
    const redis = makeMockRedis({ eval: () => Promise.resolve(1) });
    const lock = new RedisDistributedLock(redis);
    await lock.release({ key: 'resource', token: 'my-token' });
    const evalFn = redis.eval as ReturnType<typeof vi.fn>;
    expect(evalFn.mock.calls[0]).toEqual(
      expect.arrayContaining([1, 'lock:resource', 'my-token']),
    );
  });

  it('release()でトークン不一致時にLockErrorを投げる', async () => {
    const redis = makeMockRedis({ eval: () => Promise.resolve(0) });
    const lock = new RedisDistributedLock(redis);
    await expect(lock.release({ key: 'resource', token: 'wrong-token' })).rejects.toThrow(LockError);
  });

  it('extend()でLuaスクリプトによるTTL延長が実行される', async () => {
    const redis = makeMockRedis({ eval: () => Promise.resolve(1) });
    const lock = new RedisDistributedLock(redis);
    await lock.extend({ key: 'resource', token: 'my-token' }, 10000);
    const evalFn = redis.eval as ReturnType<typeof vi.fn>;
    expect(evalFn.mock.calls[0]).toEqual(
      expect.arrayContaining([1, 'lock:resource', 'my-token', '10000']),
    );
  });

  it('extend()でトークン不一致時にLockErrorを投げる', async () => {
    const redis = makeMockRedis({ eval: () => Promise.resolve(0) });
    const lock = new RedisDistributedLock(redis);
    await expect(lock.extend({ key: 'resource', token: 'wrong' }, 5000)).rejects.toThrow(LockError);
  });

  it('isLocked()がロック状態を返す', async () => {
    const redis = makeMockRedis({ exists: () => Promise.resolve(1) });
    const lock = new RedisDistributedLock(redis);
    expect(await lock.isLocked('resource')).toBe(true);
  });

  it('isLocked()がロックなし状態を返す', async () => {
    const redis = makeMockRedis({ exists: () => Promise.resolve(0) });
    const lock = new RedisDistributedLock(redis);
    expect(await lock.isLocked('resource')).toBe(false);
  });

  it('カスタムキープレフィックスが適用される', async () => {
    const redis = makeMockRedis({ set: () => Promise.resolve('OK') });
    const lock = new RedisDistributedLock(redis, 'myapp:lock');
    await lock.acquire('resource', 5000);
    const setFn = redis.set as ReturnType<typeof vi.fn>;
    expect(setFn.mock.calls[0][0]).toBe('myapp:lock:resource');
  });
});
