import 'package:uuid/uuid.dart';

import 'error.dart';
import 'envelope.dart';

abstract class EventStore {
  Future<int> append(
    String streamId,
    List<NewEvent> events, {
    int? expectedVersion,
  });
  Future<List<EventEnvelope>> load(String streamId);
  Future<List<EventEnvelope>> loadFrom(String streamId, int fromVersion);
  Future<bool> exists(String streamId);
  Future<int> currentVersion(String streamId);
}

class InMemoryEventStore implements EventStore {
  final Map<String, List<EventEnvelope>> _streams = {};
  final _uuid = const Uuid();

  @override
  Future<int> append(
    String streamId,
    List<NewEvent> events, {
    int? expectedVersion,
  }) async {
    _streams.putIfAbsent(streamId, () => []);
    final stream = _streams[streamId]!;
    final current = stream.isNotEmpty ? stream.last.version : 0;

    if (expectedVersion != null && expectedVersion != current) {
      throw VersionConflictError(expectedVersion, current);
    }

    var version = current;
    for (final event in events) {
      version++;
      stream.add(EventEnvelope(
        eventId: _uuid.v4(),
        streamId: streamId,
        version: version,
        eventType: event.eventType,
        payload: event.payload,
        metadata: event.metadata,
        recordedAt: DateTime.now(),
      ));
    }
    return version;
  }

  @override
  Future<List<EventEnvelope>> load(String streamId) async {
    return List.unmodifiable(_streams[streamId] ?? []);
  }

  @override
  Future<List<EventEnvelope>> loadFrom(
      String streamId, int fromVersion) async {
    final stream = _streams[streamId] ?? [];
    return stream.where((e) => e.version >= fromVersion).toList();
  }

  @override
  Future<bool> exists(String streamId) async {
    return _streams.containsKey(streamId);
  }

  @override
  Future<int> currentVersion(String streamId) async {
    final stream = _streams[streamId];
    if (stream == null || stream.isEmpty) return 0;
    return stream.last.version;
  }
}
