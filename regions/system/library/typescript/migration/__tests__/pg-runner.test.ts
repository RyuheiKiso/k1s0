import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PgMigrationRunner } from '../src/pg-runner.js';
import type { Pool, PoolClient, QueryResult } from 'pg';

vi.mock('node:fs/promises', () => ({
  readdir: vi.fn(),
  readFile: vi.fn(),
}));

import { readdir, readFile } from 'node:fs/promises';

function makeQueryResult<T>(rows: T[]): QueryResult<T> {
  return { rows, rowCount: rows.length, command: '', oid: 0, fields: [] };
}

function makeMockClient(queryResponses: Map<string, QueryResult<unknown>>): PoolClient {
  return {
    query: vi.fn().mockImplementation((text: string) => {
      for (const [pattern, result] of queryResponses) {
        if (text.includes(pattern)) return Promise.resolve(result);
      }
      return Promise.resolve(makeQueryResult([]));
    }),
    release: vi.fn(),
  } as unknown as PoolClient;
}

function makeMockPool(
  queryFn: (text: string) => QueryResult<unknown>,
  clientQueryFn?: (text: string) => QueryResult<unknown>,
): Pool {
  const client = {
    query: vi.fn().mockImplementation((text: string) =>
      Promise.resolve((clientQueryFn ?? queryFn)(text)),
    ),
    release: vi.fn(),
  } as unknown as PoolClient;
  return {
    connect: vi.fn().mockResolvedValue(client),
    query: vi.fn().mockImplementation((text: string) =>
      Promise.resolve(queryFn(text)),
    ),
    end: vi.fn().mockResolvedValue(undefined),
  } as unknown as Pool;
}

const migrationConfig = {
  migrationsDir: '/fake/migrations',
  databaseUrl: 'postgres://localhost/test',
};

describe('PgMigrationRunner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('runUp()', () => {
    it('未適用のマイグレーションを実行する', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([
        '20240101000001_create_users.up.sql',
        '20240101000001_create_users.down.sql',
      ]);
      (readFile as ReturnType<typeof vi.fn>).mockResolvedValue('CREATE TABLE users (id INT);');

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version FROM')) return makeQueryResult([]);
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const report = await runner.runUp();

      expect(report.appliedCount).toBe(1);
      expect(report.errors).toHaveLength(0);
    });

    it('適用済みマイグレーションをスキップする', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([
        '20240101000001_create_users.up.sql',
      ]);
      (readFile as ReturnType<typeof vi.fn>).mockResolvedValue('CREATE TABLE users (id INT);');

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version FROM')) {
          return makeQueryResult([{ version: '20240101000001' }]);
        }
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const report = await runner.runUp();

      expect(report.appliedCount).toBe(0);
    });

    it('ファイルが存在しない場合は0件を返す', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('ENOENT'));

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version FROM')) return makeQueryResult([]);
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const report = await runner.runUp();

      expect(report.appliedCount).toBe(0);
    });
  });

  describe('runDown()', () => {
    it('指定ステップ数だけロールバックする', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([
        '20240101000001_create_users.up.sql',
        '20240101000001_create_users.down.sql',
        '20240101000002_add_email.up.sql',
        '20240101000002_add_email.down.sql',
      ]);
      (readFile as ReturnType<typeof vi.fn>).mockImplementation((path: string) => {
        if ((path as string).includes('.up.')) return Promise.resolve('CREATE TABLE');
        return Promise.resolve('DROP TABLE');
      });

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version FROM')) {
          return makeQueryResult([
            { version: '20240101000002' },
            { version: '20240101000001' },
          ]);
        }
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const report = await runner.runDown(1);

      expect(report.appliedCount).toBe(1);
      expect(report.errors).toHaveLength(0);
    });
  });

  describe('status()', () => {
    it('全マイグレーションのステータスを返す', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([
        '20240101000001_create_users.up.sql',
        '20240101000002_add_email.up.sql',
      ]);
      (readFile as ReturnType<typeof vi.fn>).mockResolvedValue('CREATE TABLE');

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version')) {
          return makeQueryResult([{ version: '20240101000001', applied_at: new Date('2024-01-01') }]);
        }
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const statuses = await runner.status();

      expect(statuses).toHaveLength(2);
      expect(statuses[0].version).toBe('20240101000001');
      expect(statuses[0].appliedAt).not.toBeNull();
      expect(statuses[1].appliedAt).toBeNull();
    });
  });

  describe('pending()', () => {
    it('未適用マイグレーション一覧を返す', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([
        '20240101000001_create_users.up.sql',
        '20240101000002_add_email.up.sql',
      ]);
      (readFile as ReturnType<typeof vi.fn>).mockResolvedValue('CREATE TABLE');

      const pool = makeMockPool((text) => {
        if (text.includes('SELECT version FROM')) {
          return makeQueryResult([{ version: '20240101000001' }]);
        }
        return makeQueryResult([]);
      });

      const runner = new PgMigrationRunner(pool, migrationConfig);
      const pending = await runner.pending();

      expect(pending).toHaveLength(1);
      expect(pending[0].version).toBe('20240101000002');
      expect(pending[0].name).toBe('add_email');
    });
  });

  describe('close()', () => {
    it('プールのendを呼ぶ', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([]);

      const pool = makeMockPool(() => makeQueryResult([]));
      const runner = new PgMigrationRunner(pool, migrationConfig);
      await runner.close();
      expect((pool.end as ReturnType<typeof vi.fn>).mock.calls).toHaveLength(1);
    });
  });

  describe('カスタムテーブル名', () => {
    it('tableName オプションが使用される', async () => {
      (readdir as ReturnType<typeof vi.fn>).mockResolvedValue([]);

      const pool = makeMockPool(() => makeQueryResult([]));
      const runner = new PgMigrationRunner(pool, {
        ...migrationConfig,
        tableName: 'custom_migrations',
      });
      await runner.runUp();

      const queryFn = pool.query as ReturnType<typeof vi.fn>;
      const calls = queryFn.mock.calls.map((c: unknown[]) => c[0] as string);
      expect(calls.some((q) => q.includes('custom_migrations'))).toBe(true);
    });
  });
});
