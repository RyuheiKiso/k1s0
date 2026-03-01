import { describe, it, expect, vi } from 'vitest';
import { PostgresDistributedLock } from '../src/postgres-lock.js';
import { LockError } from '../src/index.js';
import type { Pool, QueryResult } from 'pg';

function makeQueryResult<T>(rows: T[]): QueryResult<T> {
  return { rows, rowCount: rows.length, command: '', oid: 0, fields: [] };
}

function makeMockPool(queryFn: (text: string, values?: unknown[]) => QueryResult<unknown>): Pool {
  return {
    query: vi.fn().mockImplementation((text: string, values?: unknown[]) =>
      Promise.resolve(queryFn(text, values)),
    ),
  } as unknown as Pool;
}

describe('PostgresDistributedLock', () => {
  it('acquire()でロックを取得できる', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_try_advisory_lock: true }]));
    const lock = new PostgresDistributedLock(pool);
    const guard = await lock.acquire('my-resource', 5000);
    expect(guard.key).toBe('my-resource');
    expect(guard.token).toBeTruthy();
  });

  it('acquire()でロックが取れない場合LockErrorを投げる', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_try_advisory_lock: false }]));
    const lock = new PostgresDistributedLock(pool);
    await expect(lock.acquire('my-resource', 5000)).rejects.toThrow(LockError);
  });

  it('acquire()でhashtext($1)のクエリが送られる', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_try_advisory_lock: true }]));
    const lock = new PostgresDistributedLock(pool);
    await lock.acquire('my-resource', 5000);
    const queryFn = pool.query as ReturnType<typeof vi.fn>;
    expect(queryFn.mock.calls[0][0]).toContain('pg_try_advisory_lock(hashtext($1))');
    expect(queryFn.mock.calls[0][1][0]).toBe('lock:my-resource');
  });

  it('release()でロックを解放できる', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_advisory_unlock: true }]));
    const lock = new PostgresDistributedLock(pool);
    await lock.release({ key: 'my-resource', token: 'some-token' });
    const queryFn = pool.query as ReturnType<typeof vi.fn>;
    expect(queryFn.mock.calls[0][0]).toContain('pg_advisory_unlock(hashtext($1))');
  });

  it('release()でロックが見つからない場合LockErrorを投げる', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_advisory_unlock: false }]));
    const lock = new PostgresDistributedLock(pool);
    await expect(
      lock.release({ key: 'my-resource', token: 'some-token' }),
    ).rejects.toThrow(LockError);
  });

  it('isLocked()がロック状態を返す', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ exists: true }]));
    const lock = new PostgresDistributedLock(pool);
    expect(await lock.isLocked('my-resource')).toBe(true);
  });

  it('isLocked()がロックなし状態を返す', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ exists: false }]));
    const lock = new PostgresDistributedLock(pool);
    expect(await lock.isLocked('my-resource')).toBe(false);
  });

  it('カスタムキープレフィックスが適用される', async () => {
    const pool = makeMockPool(() => makeQueryResult([{ pg_try_advisory_lock: true }]));
    const lock = new PostgresDistributedLock(pool, 'myapp:lock');
    await lock.acquire('resource', 5000);
    const queryFn = pool.query as ReturnType<typeof vi.fn>;
    expect(queryFn.mock.calls[0][1][0]).toBe('myapp:lock:resource');
  });
});
