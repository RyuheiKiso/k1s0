export type StreamId = string;

export interface EventEnvelope {
  eventId: string;
  streamId: string;
  version: number;
  eventType: string;
  payload: unknown;
  metadata?: unknown;
  recordedAt: Date;
}

export class VersionConflictError extends Error {
  constructor(
    public readonly expected: number,
    public readonly actual: number,
  ) {
    super(`version conflict: expected=${expected}, actual=${actual}`);
    this.name = 'VersionConflictError';
  }
}

export interface EventStore {
  append(
    streamId: StreamId,
    events: Omit<EventEnvelope, 'eventId' | 'version' | 'recordedAt'>[],
    expectedVersion?: number,
  ): Promise<number>;
  load(streamId: StreamId): Promise<EventEnvelope[]>;
  loadFrom(streamId: StreamId, fromVersion: number): Promise<EventEnvelope[]>;
  exists(streamId: StreamId): Promise<boolean>;
  currentVersion(streamId: StreamId): Promise<number>;
}

export interface Snapshot {
  streamId: string;
  version: number;
  state: unknown;
  createdAt: Date;
}

export interface SnapshotStore {
  saveSnapshot(snapshot: Snapshot): Promise<void>;
  loadSnapshot(streamId: StreamId): Promise<Snapshot | null>;
}

export class InMemoryEventStore implements EventStore {
  private streams = new Map<string, EventEnvelope[]>();

  async append(
    streamId: StreamId,
    events: Omit<EventEnvelope, 'eventId' | 'version' | 'recordedAt'>[],
    expectedVersion?: number,
  ): Promise<number> {
    if (!this.streams.has(streamId)) {
      this.streams.set(streamId, []);
    }
    const stream = this.streams.get(streamId)!;
    const currentVersion = stream.length > 0 ? stream[stream.length - 1].version : 0;

    if (expectedVersion !== undefined && expectedVersion !== currentVersion) {
      throw new VersionConflictError(expectedVersion, currentVersion);
    }

    let version = currentVersion;
    for (const event of events) {
      version++;
      stream.push({
        eventId: crypto.randomUUID(),
        streamId,
        version,
        eventType: event.eventType,
        payload: event.payload,
        metadata: event.metadata,
        recordedAt: new Date(),
      });
    }
    return version;
  }

  async load(streamId: StreamId): Promise<EventEnvelope[]> {
    return [...(this.streams.get(streamId) ?? [])];
  }

  async loadFrom(streamId: StreamId, fromVersion: number): Promise<EventEnvelope[]> {
    const stream = this.streams.get(streamId) ?? [];
    return stream.filter((e) => e.version >= fromVersion);
  }

  async exists(streamId: StreamId): Promise<boolean> {
    return this.streams.has(streamId);
  }

  async currentVersion(streamId: StreamId): Promise<number> {
    const stream = this.streams.get(streamId);
    if (!stream || stream.length === 0) return 0;
    return stream[stream.length - 1].version;
  }
}

export { PostgresEventStore } from './postgres-store.js';
export { PostgresSnapshotStore } from './postgres-snapshot.js';

export class InMemorySnapshotStore implements SnapshotStore {
  private snapshots = new Map<string, Snapshot>();

  async saveSnapshot(snapshot: Snapshot): Promise<void> {
    this.snapshots.set(snapshot.streamId, { ...snapshot });
  }

  async loadSnapshot(streamId: StreamId): Promise<Snapshot | null> {
    return this.snapshots.get(streamId) ?? null;
  }
}
