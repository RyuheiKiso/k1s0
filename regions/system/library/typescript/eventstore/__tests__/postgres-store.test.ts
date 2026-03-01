import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PostgresEventStore } from '../src/postgres-store.js';
import { PostgresSnapshotStore } from '../src/postgres-snapshot.js';
import { VersionConflictError } from '../src/index.js';
import type { Pool, PoolClient, QueryResult } from 'pg';

function makeQueryResult<T>(rows: T[]): QueryResult<T> {
  return { rows, rowCount: rows.length, command: '', oid: 0, fields: [] };
}

function makeMockClient(queryFn: (text: string, values?: unknown[]) => QueryResult<unknown>) {
  return {
    query: vi.fn().mockImplementation((text: string, values?: unknown[]) =>
      Promise.resolve(queryFn(text, values)),
    ),
    release: vi.fn(),
  } as unknown as PoolClient;
}

function makeMockPool(
  clientQueryFn: (text: string, values?: unknown[]) => QueryResult<unknown>,
  poolQueryFn?: (text: string, values?: unknown[]) => QueryResult<unknown>,
): Pool {
  const client = makeMockClient(clientQueryFn);
  return {
    connect: vi.fn().mockResolvedValue(client),
    query: vi.fn().mockImplementation((text: string, values?: unknown[]) =>
      Promise.resolve((poolQueryFn ?? clientQueryFn)(text, values)),
    ),
  } as unknown as Pool;
}

describe('PostgresEventStore', () => {
  it('migrate()がCREATE TABLEクエリを実行する', async () => {
    const pool = {
      query: vi.fn().mockResolvedValue(makeQueryResult([])),
    } as unknown as Pool;
    const store = new PostgresEventStore(pool);
    await store.migrate();
    expect((pool.query as ReturnType<typeof vi.fn>).mock.calls[0][0]).toContain('CREATE TABLE IF NOT EXISTS events');
  });

  describe('append()', () => {
    it('イベントを挿入してバージョンを返す', async () => {
      let callCount = 0;
      const clientQueryFn = (text: string) => {
        if (text.includes('BEGIN') || text.includes('COMMIT') || text.includes('ROLLBACK')) {
          return makeQueryResult([]);
        }
        if (text.includes('MAX(version)')) {
          return makeQueryResult([{ version: '0' }]);
        }
        if (text.includes('INSERT INTO events')) {
          callCount++;
          return makeQueryResult([]);
        }
        return makeQueryResult([]);
      };
      const pool = makeMockPool(clientQueryFn);
      const store = new PostgresEventStore(pool);

      const version = await store.append('stream-1', [
        { streamId: 'stream-1', eventType: 'TestEvent', payload: { data: 1 } },
        { streamId: 'stream-1', eventType: 'TestEvent2', payload: { data: 2 } },
      ]);

      expect(version).toBe(2);
      expect(callCount).toBe(2);
    });

    it('expectedVersionが不一致の場合VersionConflictErrorを投げる', async () => {
      const clientQueryFn = (text: string) => {
        if (text.includes('BEGIN') || text.includes('COMMIT') || text.includes('ROLLBACK')) {
          return makeQueryResult([]);
        }
        if (text.includes('MAX(version)')) {
          return makeQueryResult([{ version: '3' }]);
        }
        return makeQueryResult([]);
      };
      const pool = makeMockPool(clientQueryFn);
      const store = new PostgresEventStore(pool);

      await expect(
        store.append('stream-1', [{ streamId: 'stream-1', eventType: 'E', payload: {} }], 1),
      ).rejects.toThrow(VersionConflictError);
    });

    it('エラー時にROLLBACKを呼ぶ', async () => {
      const mockClient = {
        query: vi.fn().mockImplementation((text: string) => {
          if (text.includes('BEGIN') || text.includes('ROLLBACK')) return Promise.resolve(makeQueryResult([]));
          if (text.includes('MAX(version)')) return Promise.resolve(makeQueryResult([{ version: '0' }]));
          if (text.includes('INSERT')) return Promise.reject(new Error('DB error'));
          return Promise.resolve(makeQueryResult([]));
        }),
        release: vi.fn(),
      } as unknown as PoolClient;
      const pool = { connect: vi.fn().mockResolvedValue(mockClient) } as unknown as Pool;
      const store = new PostgresEventStore(pool);

      await expect(
        store.append('stream-1', [{ streamId: 'stream-1', eventType: 'E', payload: {} }]),
      ).rejects.toThrow('DB error');

      const calls = (mockClient.query as ReturnType<typeof vi.fn>).mock.calls.map((c: unknown[]) => c[0]);
      expect(calls).toContain('ROLLBACK');
    });
  });

  describe('load()', () => {
    it('ストリームのイベントを返す', async () => {
      const rows = [
        { id: 'uuid-1', stream_id: 'stream-1', version: '1', event_type: 'TestEvent', payload: { data: 1 }, metadata: {}, recorded_at: new Date('2024-01-01') },
      ];
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult(rows)),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);

      const events = await store.load('stream-1');
      expect(events).toHaveLength(1);
      expect(events[0].eventId).toBe('uuid-1');
      expect(events[0].version).toBe(1);
      expect(events[0].eventType).toBe('TestEvent');
    });

    it('存在しないストリームは空配列を返す', async () => {
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult([])),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      const events = await store.load('nonexistent');
      expect(events).toHaveLength(0);
    });
  });

  describe('loadFrom()', () => {
    it('指定バージョン以降のイベントを返す', async () => {
      const rows = [
        { id: 'uuid-2', stream_id: 'stream-1', version: '2', event_type: 'E2', payload: {}, metadata: {}, recorded_at: new Date() },
        { id: 'uuid-3', stream_id: 'stream-1', version: '3', event_type: 'E3', payload: {}, metadata: {}, recorded_at: new Date() },
      ];
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult(rows)),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      const events = await store.loadFrom('stream-1', 2);
      expect(events).toHaveLength(2);
      expect(events[0].version).toBe(2);
    });
  });

  describe('exists()', () => {
    it('イベントがある場合trueを返す', async () => {
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult([{ count: '1' }])),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      expect(await store.exists('stream-1')).toBe(true);
    });

    it('イベントがない場合falseを返す', async () => {
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult([{ count: '0' }])),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      expect(await store.exists('stream-empty')).toBe(false);
    });
  });

  describe('currentVersion()', () => {
    it('現在のバージョンを返す', async () => {
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult([{ version: '5' }])),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      expect(await store.currentVersion('stream-1')).toBe(5);
    });

    it('イベントがない場合0を返す', async () => {
      const pool = {
        query: vi.fn().mockResolvedValue(makeQueryResult([{ version: '0' }])),
      } as unknown as Pool;
      const store = new PostgresEventStore(pool);
      expect(await store.currentVersion('empty')).toBe(0);
    });
  });
});

describe('PostgresSnapshotStore', () => {
  it('migrate()がCREATE TABLEクエリを実行する', async () => {
    const pool = {
      query: vi.fn().mockResolvedValue(makeQueryResult([])),
    } as unknown as Pool;
    const store = new PostgresSnapshotStore(pool);
    await store.migrate();
    expect((pool.query as ReturnType<typeof vi.fn>).mock.calls[0][0]).toContain('CREATE TABLE IF NOT EXISTS snapshots');
  });

  it('saveSnapshot()がUPSERTクエリを実行する', async () => {
    const pool = {
      query: vi.fn().mockResolvedValue(makeQueryResult([])),
    } as unknown as Pool;
    const store = new PostgresSnapshotStore(pool);
    await store.saveSnapshot({ streamId: 'stream-1', version: 5, state: { x: 1 }, createdAt: new Date() });
    const call = (pool.query as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(call[0]).toContain('ON CONFLICT');
    expect(call[1][0]).toBe('stream-1');
    expect(call[1][1]).toBe(5);
  });

  it('loadSnapshot()がスナップショットを返す', async () => {
    const rows = [{
      stream_id: 'stream-1',
      version: '5',
      state: { x: 1 },
      created_at: new Date('2024-01-01'),
    }];
    const pool = {
      query: vi.fn().mockResolvedValue(makeQueryResult(rows)),
    } as unknown as Pool;
    const store = new PostgresSnapshotStore(pool);
    const snap = await store.loadSnapshot('stream-1');
    expect(snap).not.toBeNull();
    expect(snap!.streamId).toBe('stream-1');
    expect(snap!.version).toBe(5);
    expect(snap!.state).toEqual({ x: 1 });
  });

  it('loadSnapshot()が存在しない場合nullを返す', async () => {
    const pool = {
      query: vi.fn().mockResolvedValue(makeQueryResult([])),
    } as unknown as Pool;
    const store = new PostgresSnapshotStore(pool);
    expect(await store.loadSnapshot('nonexistent')).toBeNull();
  });
});
