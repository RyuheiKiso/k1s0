export { PgMigrationRunner } from './pg-runner.js';

import { createHash } from 'node:crypto';

export interface MigrationStatus {
  version: string;
  name: string;
  appliedAt: Date | null;
  checksum: string;
}

export interface PendingMigration {
  version: string;
  name: string;
}

export interface MigrationReport {
  appliedCount: number;
  elapsedMs: number;
  errors: Error[];
}

export interface MigrationConfig {
  migrationsDir: string;
  databaseUrl: string;
  tableName?: string;
}

export interface MigrationRunner {
  runUp(): Promise<MigrationReport>;
  runDown(steps: number): Promise<MigrationReport>;
  status(): Promise<MigrationStatus[]>;
  pending(): Promise<PendingMigration[]>;
}

export type MigrationDirection = 'up' | 'down';

export interface ParsedMigration {
  version: string;
  name: string;
  direction: MigrationDirection;
}

export function parseFilename(filename: string): ParsedMigration | null {
  if (!filename.endsWith('.sql')) return null;
  const stem = filename.slice(0, -4);

  let direction: MigrationDirection;
  let rest: string;

  if (stem.endsWith('.up')) {
    direction = 'up';
    rest = stem.slice(0, -3);
  } else if (stem.endsWith('.down')) {
    direction = 'down';
    rest = stem.slice(0, -5);
  } else {
    return null;
  }

  const idx = rest.indexOf('_');
  if (idx <= 0 || idx >= rest.length - 1) return null;

  const version = rest.slice(0, idx);
  const name = rest.slice(idx + 1);

  return { version, name, direction };
}

export function checksum(content: string): string {
  return createHash('sha256').update(content).digest('hex');
}

export class MigrationError extends Error {
  constructor(
    message: string,
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = 'MigrationError';
  }
}

interface MigrationEntry {
  version: string;
  name: string;
  content: string;
}

export class InMemoryMigrationRunner implements MigrationRunner {
  private readonly config: MigrationConfig;
  private readonly upMigrations: MigrationEntry[];
  private readonly downMigrations: Map<string, MigrationEntry>;
  private applied: MigrationStatus[] = [];

  constructor(
    config: MigrationConfig,
    ups: MigrationEntry[],
    downs: MigrationEntry[],
  ) {
    this.config = config;
    this.upMigrations = [...ups].sort((a, b) =>
      a.version.localeCompare(b.version),
    );
    this.downMigrations = new Map(downs.map((d) => [d.version, d]));
  }

  async runUp(): Promise<MigrationReport> {
    const start = Date.now();
    const appliedVersions = new Set(this.applied.map((s) => s.version));
    let count = 0;

    for (const mf of this.upMigrations) {
      if (appliedVersions.has(mf.version)) continue;
      this.applied.push({
        version: mf.version,
        name: mf.name,
        appliedAt: new Date(),
        checksum: checksum(mf.content),
      });
      count++;
    }

    return { appliedCount: count, elapsedMs: Date.now() - start, errors: [] };
  }

  async runDown(steps: number): Promise<MigrationReport> {
    const start = Date.now();
    let count = 0;

    for (let i = 0; i < steps; i++) {
      if (this.applied.length === 0) break;
      this.applied.pop();
      count++;
    }

    return { appliedCount: count, elapsedMs: Date.now() - start, errors: [] };
  }

  async status(): Promise<MigrationStatus[]> {
    const appliedMap = new Map(
      this.applied.map((s) => [s.version, s]),
    );
    return this.upMigrations.map((mf) => {
      const cs = checksum(mf.content);
      const applied = appliedMap.get(mf.version);
      return {
        version: mf.version,
        name: mf.name,
        appliedAt: applied?.appliedAt ?? null,
        checksum: cs,
      };
    });
  }

  async pending(): Promise<PendingMigration[]> {
    const appliedVersions = new Set(this.applied.map((s) => s.version));
    return this.upMigrations
      .filter((mf) => !appliedVersions.has(mf.version))
      .map((mf) => ({ version: mf.version, name: mf.name }));
  }
}
