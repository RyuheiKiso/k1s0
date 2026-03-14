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

    test('appendが新しいバージョンを返すこと', () async {
      final version = await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'Created', payload: {'x': 1}),
      ]);
      expect(version, equals(1));
    });

    test('expectedVersionの競合時にVersionConflictErrorをスローすること', () async {
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

    test('loadFromが指定バージョン以降のイベントを返すこと', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
        const NewEvent(streamId: 's1', eventType: 'E3'),
      ]);
      final events = await store.loadFrom('s1', 2);
      expect(events, hasLength(2));
      expect(events.first.version, equals(2));
    });

    test('existsが追記前にfalse、追記後にtrueを返すこと', () async {
      expect(await store.exists('s1'), isFalse);
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
      ]);
      expect(await store.exists('s1'), isTrue);
    });

    test('currentVersionが空のストリームに対して0を返すこと', () async {
      expect(await store.currentVersion('nope'), equals(0));
    });
  });

  group('PostgresSnapshotStore contract (via InMemory)', () {
    late InMemorySnapshotStore store;

    setUp(() {
      store = InMemorySnapshotStore();
    });

    test('loadSnapshotが未知のストリームに対してnullを返すこと', () async {
      expect(await store.loadSnapshot('unknown'), isNull);
    });

    test('saveSnapshotとloadSnapshotのラウンドトリップが正常に動作すること', () async {
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

    test('saveSnapshotが既存のスナップショットを上書きすること', () async {
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
