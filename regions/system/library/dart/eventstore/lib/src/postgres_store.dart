import 'package:postgres/postgres.dart';
import 'package:uuid/uuid.dart';

import 'envelope.dart';
import 'error.dart';
import 'snapshot.dart';
import 'store.dart';

class PostgresEventStore implements EventStore {
  final Connection _conn;
  final _uuid = const Uuid();

  PostgresEventStore(this._conn);

  Future<void> migrate() async {
    await _conn.execute('''
      CREATE TABLE IF NOT EXISTS events (
        id UUID PRIMARY KEY,
        stream_id TEXT NOT NULL,
        version BIGINT NOT NULL,
        event_type TEXT NOT NULL,
        payload JSONB,
        metadata JSONB,
        recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE (stream_id, version)
      )
    ''');
  }

  @override
  Future<int> append(
    String streamId,
    List<NewEvent> events, {
    int? expectedVersion,
  }) async {
    if (events.isEmpty) return await currentVersion(streamId);

    return await _conn.runTx((tx) async {
      final rows = await tx.execute(
        Sql.named(
            'SELECT COALESCE(MAX(version), 0) AS ver FROM events WHERE stream_id = @streamId'),
        parameters: {'streamId': streamId},
      );
      final current = (rows.first[0] as int?) ?? 0;

      if (expectedVersion != null && expectedVersion != current) {
        throw VersionConflictError(expectedVersion, current);
      }

      var version = current;
      for (final event in events) {
        version++;
        await tx.execute(
          Sql.named(
              'INSERT INTO events (id, stream_id, version, event_type, payload, metadata, recorded_at) '
              'VALUES (@id, @streamId, @version, @eventType, @payload, @metadata, NOW())'),
          parameters: {
            'id': _uuid.v4(),
            'streamId': streamId,
            'version': version,
            'eventType': event.eventType,
            'payload': event.payload,
            'metadata': event.metadata,
          },
        );
      }
      return version;
    });
  }

  @override
  Future<List<EventEnvelope>> load(String streamId) async {
    final rows = await _conn.execute(
      Sql.named(
          'SELECT id, stream_id, version, event_type, payload, metadata, recorded_at '
          'FROM events WHERE stream_id = @streamId ORDER BY version ASC'),
      parameters: {'streamId': streamId},
    );
    return rows.map(_rowToEnvelope).toList();
  }

  @override
  Future<List<EventEnvelope>> loadFrom(String streamId, int fromVersion) async {
    final rows = await _conn.execute(
      Sql.named(
          'SELECT id, stream_id, version, event_type, payload, metadata, recorded_at '
          'FROM events WHERE stream_id = @streamId AND version >= @fromVersion ORDER BY version ASC'),
      parameters: {'streamId': streamId, 'fromVersion': fromVersion},
    );
    return rows.map(_rowToEnvelope).toList();
  }

  @override
  Future<bool> exists(String streamId) async {
    final rows = await _conn.execute(
      Sql.named('SELECT 1 FROM events WHERE stream_id = @streamId LIMIT 1'),
      parameters: {'streamId': streamId},
    );
    return rows.isNotEmpty;
  }

  @override
  Future<int> currentVersion(String streamId) async {
    final rows = await _conn.execute(
      Sql.named(
          'SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = @streamId'),
      parameters: {'streamId': streamId},
    );
    return (rows.first[0] as int?) ?? 0;
  }

  EventEnvelope _rowToEnvelope(ResultRow row) {
    return EventEnvelope(
      eventId: row[0] as String,
      streamId: row[1] as String,
      version: row[2] as int,
      eventType: row[3] as String,
      payload: row[4],
      metadata: row[5],
      recordedAt: row[6] as DateTime,
    );
  }
}

class PostgresSnapshotStore implements SnapshotStore {
  final Connection _conn;

  PostgresSnapshotStore(this._conn);

  Future<void> migrate() async {
    await _conn.execute('''
      CREATE TABLE IF NOT EXISTS snapshots (
        stream_id TEXT PRIMARY KEY,
        version BIGINT NOT NULL,
        state JSONB,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    ''');
  }

  @override
  Future<void> saveSnapshot(Snapshot snapshot) async {
    await _conn.execute(
      Sql.named(
          'INSERT INTO snapshots (stream_id, version, state, created_at) '
          'VALUES (@streamId, @version, @state, NOW()) '
          'ON CONFLICT (stream_id) DO UPDATE SET version = EXCLUDED.version, state = EXCLUDED.state, created_at = EXCLUDED.created_at'),
      parameters: {
        'streamId': snapshot.streamId,
        'version': snapshot.version,
        'state': snapshot.state,
      },
    );
  }

  @override
  Future<Snapshot?> loadSnapshot(String streamId) async {
    final rows = await _conn.execute(
      Sql.named(
          'SELECT stream_id, version, state, created_at FROM snapshots WHERE stream_id = @streamId'),
      parameters: {'streamId': streamId},
    );
    if (rows.isEmpty) return null;
    final row = rows.first;
    return Snapshot(
      streamId: row[0] as String,
      version: row[1] as int,
      state: row[2],
      createdAt: row[3] as DateTime,
    );
  }
}
