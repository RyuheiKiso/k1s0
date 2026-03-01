import { randomUUID } from 'crypto';
import type { Pool } from 'pg';
import {
  type EventEnvelope,
  type EventStore,
  type StreamId,
  VersionConflictError,
} from './index.js';

const CREATE_EVENTS_TABLE = `
CREATE TABLE IF NOT EXISTS events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  stream_id TEXT NOT NULL,
  version BIGINT NOT NULL,
  event_type TEXT NOT NULL,
  payload JSONB NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}',
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (stream_id, version)
);
CREATE INDEX IF NOT EXISTS idx_events_stream_id ON events (stream_id, version);
`;

export class PostgresEventStore implements EventStore {
  constructor(private readonly pool: Pool) {}

  async migrate(): Promise<void> {
    await this.pool.query(CREATE_EVENTS_TABLE);
  }

  async append(
    streamId: StreamId,
    events: Omit<EventEnvelope, 'eventId' | 'version' | 'recordedAt'>[],
    expectedVersion?: number,
  ): Promise<number> {
    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');

      const versionResult = await client.query<{ version: string }>(
        'SELECT COALESCE(MAX(version), 0) AS version FROM events WHERE stream_id = $1 FOR UPDATE',
        [streamId],
      );
      const currentVersion = Number(versionResult.rows[0].version);

      if (expectedVersion !== undefined && expectedVersion !== currentVersion) {
        throw new VersionConflictError(expectedVersion, currentVersion);
      }

      let version = currentVersion;
      for (const event of events) {
        version++;
        await client.query(
          `INSERT INTO events (id, stream_id, version, event_type, payload, metadata)
           VALUES ($1, $2, $3, $4, $5, $6)`,
          [
            randomUUID(),
            streamId,
            version,
            event.eventType,
            JSON.stringify(event.payload),
            JSON.stringify(event.metadata ?? {}),
          ],
        );
      }

      await client.query('COMMIT');
      return version;
    } catch (err) {
      await client.query('ROLLBACK');
      throw err;
    } finally {
      client.release();
    }
  }

  async load(streamId: StreamId): Promise<EventEnvelope[]> {
    const result = await this.pool.query<{
      id: string;
      stream_id: string;
      version: string;
      event_type: string;
      payload: unknown;
      metadata: unknown;
      recorded_at: Date;
    }>(
      'SELECT id, stream_id, version, event_type, payload, metadata, recorded_at FROM events WHERE stream_id = $1 ORDER BY version ASC',
      [streamId],
    );
    return result.rows.map((row) => ({
      eventId: row.id,
      streamId: row.stream_id,
      version: Number(row.version),
      eventType: row.event_type,
      payload: row.payload,
      metadata: row.metadata,
      recordedAt: row.recorded_at,
    }));
  }

  async loadFrom(streamId: StreamId, fromVersion: number): Promise<EventEnvelope[]> {
    const result = await this.pool.query<{
      id: string;
      stream_id: string;
      version: string;
      event_type: string;
      payload: unknown;
      metadata: unknown;
      recorded_at: Date;
    }>(
      'SELECT id, stream_id, version, event_type, payload, metadata, recorded_at FROM events WHERE stream_id = $1 AND version >= $2 ORDER BY version ASC',
      [streamId, fromVersion],
    );
    return result.rows.map((row) => ({
      eventId: row.id,
      streamId: row.stream_id,
      version: Number(row.version),
      eventType: row.event_type,
      payload: row.payload,
      metadata: row.metadata,
      recordedAt: row.recorded_at,
    }));
  }

  async exists(streamId: StreamId): Promise<boolean> {
    const result = await this.pool.query<{ count: string }>(
      'SELECT COUNT(*) AS count FROM events WHERE stream_id = $1 LIMIT 1',
      [streamId],
    );
    return Number(result.rows[0].count) > 0;
  }

  async currentVersion(streamId: StreamId): Promise<number> {
    const result = await this.pool.query<{ version: string }>(
      'SELECT COALESCE(MAX(version), 0) AS version FROM events WHERE stream_id = $1',
      [streamId],
    );
    return Number(result.rows[0].version);
  }
}
