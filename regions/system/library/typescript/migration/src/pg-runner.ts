import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { Pool } from 'pg';
import {
  checksum,
  MigrationError,
  parseFilename,
  type MigrationConfig,
  type MigrationReport,
  type MigrationRunner,
  type MigrationStatus,
  type PendingMigration,
} from './index.js';

export class PgMigrationRunner implements MigrationRunner {
  private readonly tableName: string;

  constructor(
    private readonly pool: Pool,
    private readonly config: MigrationConfig,
  ) {
    this.tableName = config.tableName ?? '_migrations';
  }

  private async ensureTable(): Promise<void> {
    await this.pool.query(`
      CREATE TABLE IF NOT EXISTS ${this.tableName} (
        version TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        checksum TEXT NOT NULL,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    `);
  }

  private async readMigrationFiles(): Promise<
    { version: string; name: string; upContent: string; downContent: string | null }[]
  > {
    const dir = this.config.migrationsDir;
    let files: string[];
    try {
      files = await readdir(dir);
    } catch {
      return [];
    }

    const upMap = new Map<string, { version: string; name: string; content: string }>();
    const downMap = new Map<string, string>();

    for (const file of files) {
      const parsed = parseFilename(file);
      if (!parsed) continue;
      const content = await readFile(join(dir, file), 'utf-8');
      if (parsed.direction === 'up') {
        upMap.set(parsed.version, { version: parsed.version, name: parsed.name, content });
      } else {
        downMap.set(parsed.version, content);
      }
    }

    return [...upMap.values()]
      .sort((a, b) => a.version.localeCompare(b.version))
      .map((up) => ({
        version: up.version,
        name: up.name,
        upContent: up.content,
        downContent: downMap.get(up.version) ?? null,
      }));
  }

  async runUp(): Promise<MigrationReport> {
    const start = Date.now();
    const errors: Error[] = [];
    let appliedCount = 0;

    await this.ensureTable();

    const migrations = await this.readMigrationFiles();
    const appliedResult = await this.pool.query<{ version: string }>(
      `SELECT version FROM ${this.tableName}`,
    );
    const appliedVersions = new Set(appliedResult.rows.map((r) => r.version));

    for (const mig of migrations) {
      if (appliedVersions.has(mig.version)) continue;

      const client = await this.pool.connect();
      try {
        await client.query('BEGIN');
        await client.query(mig.upContent);
        await client.query(
          `INSERT INTO ${this.tableName} (version, name, checksum) VALUES ($1, $2, $3)`,
          [mig.version, mig.name, checksum(mig.upContent)],
        );
        await client.query('COMMIT');
        appliedCount++;
      } catch (err) {
        await client.query('ROLLBACK');
        errors.push(new MigrationError(`failed to apply ${mig.version}: ${String(err)}`, err instanceof Error ? err : undefined));
      } finally {
        client.release();
      }
    }

    return { appliedCount, elapsedMs: Date.now() - start, errors };
  }

  async runDown(steps: number): Promise<MigrationReport> {
    const start = Date.now();
    const errors: Error[] = [];
    let appliedCount = 0;

    await this.ensureTable();

    const migrations = await this.readMigrationFiles();
    const appliedResult = await this.pool.query<{ version: string }>(
      `SELECT version FROM ${this.tableName} ORDER BY version DESC`,
    );
    const appliedVersions = appliedResult.rows.map((r) => r.version);

    for (let i = 0; i < steps && i < appliedVersions.length; i++) {
      const version = appliedVersions[i];
      const mig = migrations.find((m) => m.version === version);

      const client = await this.pool.connect();
      try {
        await client.query('BEGIN');
        if (mig?.downContent) {
          await client.query(mig.downContent);
        }
        await client.query(`DELETE FROM ${this.tableName} WHERE version = $1`, [version]);
        await client.query('COMMIT');
        appliedCount++;
      } catch (err) {
        await client.query('ROLLBACK');
        errors.push(new MigrationError(`failed to rollback ${version}: ${String(err)}`, err instanceof Error ? err : undefined));
      } finally {
        client.release();
      }
    }

    return { appliedCount, elapsedMs: Date.now() - start, errors };
  }

  async status(): Promise<MigrationStatus[]> {
    await this.ensureTable();

    const migrations = await this.readMigrationFiles();
    const appliedResult = await this.pool.query<{ version: string; applied_at: Date }>(
      `SELECT version, applied_at FROM ${this.tableName}`,
    );
    const appliedMap = new Map(appliedResult.rows.map((r) => [r.version, r.applied_at]));

    return migrations.map((mig) => ({
      version: mig.version,
      name: mig.name,
      appliedAt: appliedMap.get(mig.version) ?? null,
      checksum: checksum(mig.upContent),
    }));
  }

  async pending(): Promise<PendingMigration[]> {
    await this.ensureTable();

    const migrations = await this.readMigrationFiles();
    const appliedResult = await this.pool.query<{ version: string }>(
      `SELECT version FROM ${this.tableName}`,
    );
    const appliedVersions = new Set(appliedResult.rows.map((r) => r.version));

    return migrations
      .filter((mig) => !appliedVersions.has(mig.version))
      .map((mig) => ({ version: mig.version, name: mig.name }));
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}
