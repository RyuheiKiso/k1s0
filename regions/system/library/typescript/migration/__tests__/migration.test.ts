import { describe, it, expect, beforeEach } from 'vitest';
import {
  InMemoryMigrationRunner,
  parseFilename,
  checksum,
  MigrationError,
} from '../src/index.js';

function createRunner() {
  const ups = [
    { version: '20240101000001', name: 'create_users', content: 'CREATE TABLE users (id INT);' },
    { version: '20240101000002', name: 'add_email', content: 'ALTER TABLE users ADD COLUMN email TEXT;' },
    { version: '20240201000001', name: 'create_orders', content: 'CREATE TABLE orders (id INT);' },
  ];
  const downs = [
    { version: '20240101000001', name: 'create_users', content: 'DROP TABLE users;' },
    { version: '20240101000002', name: 'add_email', content: 'ALTER TABLE users DROP COLUMN email;' },
    { version: '20240201000001', name: 'create_orders', content: 'DROP TABLE orders;' },
  ];
  return new InMemoryMigrationRunner(
    { migrationsDir: '.', databaseUrl: 'memory://' },
    ups,
    downs,
  );
}

describe('parseFilename', () => {
  it('should parse up migration', () => {
    const result = parseFilename('20240101000001_create_users.up.sql');
    expect(result).toEqual({
      version: '20240101000001',
      name: 'create_users',
      direction: 'up',
    });
  });

  it('should parse down migration', () => {
    const result = parseFilename('20240101000001_create_users.down.sql');
    expect(result).toEqual({
      version: '20240101000001',
      name: 'create_users',
      direction: 'down',
    });
  });

  it('should return null for invalid filenames', () => {
    expect(parseFilename('invalid.sql')).toBeNull();
    expect(parseFilename('no_direction.sql')).toBeNull();
    expect(parseFilename('_.up.sql')).toBeNull();
  });
});

describe('checksum', () => {
  it('should be deterministic', () => {
    const content = 'CREATE TABLE users (id SERIAL PRIMARY KEY);';
    expect(checksum(content)).toBe(checksum(content));
  });

  it('should differ for different content', () => {
    expect(checksum('CREATE TABLE users;')).not.toBe(checksum('CREATE TABLE orders;'));
  });
});

describe('MigrationError', () => {
  it('should create error with message', () => {
    const err = new MigrationError('test error');
    expect(err.message).toBe('test error');
    expect(err.name).toBe('MigrationError');
  });

  it('should create error with cause', () => {
    const cause = new Error('root cause');
    const err = new MigrationError('test error', cause);
    expect(err.cause).toBe(cause);
  });
});

describe('InMemoryMigrationRunner', () => {
  let runner: InstanceType<typeof InMemoryMigrationRunner>;

  beforeEach(() => {
    runner = createRunner();
  });

  it('runUp applies all migrations', async () => {
    const report = await runner.runUp();
    expect(report.appliedCount).toBe(3);
    expect(report.errors).toHaveLength(0);
  });

  it('runUp is idempotent', async () => {
    await runner.runUp();
    const report = await runner.runUp();
    expect(report.appliedCount).toBe(0);
  });

  it('runDown rolls back one step', async () => {
    await runner.runUp();
    const report = await runner.runDown(1);
    expect(report.appliedCount).toBe(1);

    const pending = await runner.pending();
    expect(pending).toHaveLength(1);
    expect(pending[0].version).toBe('20240201000001');
  });

  it('runDown rolls back multiple steps', async () => {
    await runner.runUp();
    const report = await runner.runDown(2);
    expect(report.appliedCount).toBe(2);

    const pending = await runner.pending();
    expect(pending).toHaveLength(2);
  });

  it('runDown handles more steps than applied', async () => {
    await runner.runUp();
    const report = await runner.runDown(10);
    expect(report.appliedCount).toBe(3);
  });

  it('status shows all pending initially', async () => {
    const statuses = await runner.status();
    expect(statuses).toHaveLength(3);
    for (const s of statuses) {
      expect(s.appliedAt).toBeNull();
    }
  });

  it('status shows all applied after runUp', async () => {
    await runner.runUp();
    const statuses = await runner.status();
    expect(statuses).toHaveLength(3);
    for (const s of statuses) {
      expect(s.appliedAt).not.toBeNull();
    }
  });

  it('pending returns all unapplied migrations', async () => {
    const pending = await runner.pending();
    expect(pending).toHaveLength(3);
    expect(pending[0].version).toBe('20240101000001');
    expect(pending[1].version).toBe('20240101000002');
    expect(pending[2].version).toBe('20240201000001');
  });

  it('pending returns empty after full apply', async () => {
    await runner.runUp();
    const pending = await runner.pending();
    expect(pending).toHaveLength(0);
  });
});
