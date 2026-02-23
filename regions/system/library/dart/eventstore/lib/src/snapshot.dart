class Snapshot {
  final String streamId;
  final int version;
  final Object? state;
  final DateTime createdAt;

  const Snapshot({
    required this.streamId,
    required this.version,
    this.state,
    required this.createdAt,
  });
}

abstract class SnapshotStore {
  Future<void> saveSnapshot(Snapshot snapshot);
  Future<Snapshot?> loadSnapshot(String streamId);
}

class InMemorySnapshotStore implements SnapshotStore {
  final Map<String, Snapshot> _snapshots = {};

  @override
  Future<void> saveSnapshot(Snapshot snapshot) async {
    _snapshots[snapshot.streamId] = snapshot;
  }

  @override
  Future<Snapshot?> loadSnapshot(String streamId) async {
    return _snapshots[streamId];
  }
}
