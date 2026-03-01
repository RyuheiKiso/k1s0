import 'package:test/test.dart';
import 'package:k1s0_eventstore/k1s0_eventstore.dart';

// These tests verify PostgresEventStore and PostgresSnapshotStore interfaces.
// Integration tests against a real database are omitted here.
// Instead, we verify that InMemory implementations cover the same contract.

void main() {
  group('PostgresEventStore contract (via InMemory)', () {
    late InMemoryEventStore store;

    setUp(() {
      store = InMemoryEventStore();
    });

    test('append returns new version', () async {
      final version = await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'Created', payload: {'x': 1}),
      ]);
      expect(version, equals(1));
    });

    test('append with expectedVersion conflict throws VersionConflictError', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'Created'),
      ]);
      expect(
        () => store.append('s1', [
          const NewEvent(streamId: 's1', eventType: 'Updated'),
        ], expectedVersion: 0),
        throwsA(isA<VersionConflictError>()),
      );
    });

    test('loadFrom returns events from specified version', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
        const NewEvent(streamId: 's1', eventType: 'E3'),
      ]);
      final events = await store.loadFrom('s1', 2);
      expect(events, hasLength(2));
      expect(events.first.version, equals(2));
    });

    test('exists returns false then true', () async {
      expect(await store.exists('s1'), isFalse);
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
      ]);
      expect(await store.exists('s1'), isTrue);
    });

    test('currentVersion returns 0 for empty stream', () async {
      expect(await store.currentVersion('nope'), equals(0));
    });
  });

  group('PostgresSnapshotStore contract (via InMemory)', () {
    late InMemorySnapshotStore store;

    setUp(() {
      store = InMemorySnapshotStore();
    });

    test('loadSnapshot returns null for unknown stream', () async {
      expect(await store.loadSnapshot('unknown'), isNull);
    });

    test('saveSnapshot and loadSnapshot round-trip', () async {
      final snap = Snapshot(
        streamId: 's1',
        version: 10,
        state: {'count': 42},
        createdAt: DateTime.now(),
      );
      await store.saveSnapshot(snap);
      final loaded = await store.loadSnapshot('s1');
      expect(loaded, isNotNull);
      expect(loaded!.version, equals(10));
      expect(loaded.state, equals({'count': 42}));
    });

    test('saveSnapshot overwrites existing snapshot', () async {
      await store.saveSnapshot(Snapshot(
        streamId: 's1',
        version: 5,
        createdAt: DateTime.now(),
      ));
      await store.saveSnapshot(Snapshot(
        streamId: 's1',
        version: 15,
        createdAt: DateTime.now(),
      ));
      final loaded = await store.loadSnapshot('s1');
      expect(loaded!.version, equals(15));
    });
  });
}
