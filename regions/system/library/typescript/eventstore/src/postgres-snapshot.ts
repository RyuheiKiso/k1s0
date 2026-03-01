import type { Pool } from 'pg';
import { type Snapshot, type SnapshotStore, type StreamId } from './index.js';

const CREATE_SNAPSHOTS_TABLE = `
CREATE TABLE IF NOT EXISTS snapshots (
  stream_id TEXT PRIMARY KEY,
  version BIGINT NOT NULL,
  state JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
`;

export class PostgresSnapshotStore implements SnapshotStore {
  constructor(private readonly pool: Pool) {}

  async migrate(): Promise<void> {
    await this.pool.query(CREATE_SNAPSHOTS_TABLE);
  }

  async saveSnapshot(snapshot: Snapshot): Promise<void> {
    await this.pool.query(
      `INSERT INTO snapshots (stream_id, version, state, created_at)
       VALUES ($1, $2, $3, $4)
       ON CONFLICT (stream_id) DO UPDATE SET version = $2, state = $3, created_at = $4`,
      [snapshot.streamId, snapshot.version, JSON.stringify(snapshot.state), snapshot.createdAt],
    );
  }

  async loadSnapshot(streamId: StreamId): Promise<Snapshot | null> {
    const result = await this.pool.query<{
      stream_id: string;
      version: string;
      state: unknown;
      created_at: Date;
    }>(
      'SELECT stream_id, version, state, created_at FROM snapshots WHERE stream_id = $1',
      [streamId],
    );
    if (result.rows.length === 0) return null;
    const row = result.rows[0];
    return {
      streamId: row.stream_id,
      version: Number(row.version),
      state: row.state,
      createdAt: row.created_at,
    };
  }
}
