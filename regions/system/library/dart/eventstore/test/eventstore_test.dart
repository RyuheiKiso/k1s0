import 'package:test/test.dart';

import 'package:k1s0_eventstore/k1s0_eventstore.dart';

void main() {
  late InMemoryEventStore store;

  setUp(() {
    store = InMemoryEventStore();
  });

  group('append/load', () {
    test('appends and loads events', () async {
      await store.append('stream-1', [
        const NewEvent(
          streamId: 'stream-1',
          eventType: 'Created',
          payload: {'name': 'test'},
        ),
      ]);
      final events = await store.load('stream-1');
      expect(events, hasLength(1));
      expect(events[0].eventType, equals('Created'));
      expect(events[0].version, equals(1));
      expect(events[0].streamId, equals('stream-1'));
      expect(events[0].eventId, isNotEmpty);
    });

    test('increments version for multiple events', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
      ]);
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E3'),
      ]);
      final events = await store.load('s1');
      expect(events, hasLength(3));
      expect(events[0].version, equals(1));
      expect(events[1].version, equals(2));
      expect(events[2].version, equals(3));
    });

    test('returns empty list for nonexistent stream', () async {
      final events = await store.load('nonexistent');
      expect(events, isEmpty);
    });
  });

  group('expectedVersion', () {
    test('succeeds with correct expected version', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
      ]);
      final version = await store.append(
        's1',
        [const NewEvent(streamId: 's1', eventType: 'E2')],
        expectedVersion: 1,
      );
      expect(version, equals(2));
    });

    test('throws VersionConflictError on mismatch', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
      ]);
      expect(
        () => store.append(
          's1',
          [const NewEvent(streamId: 's1', eventType: 'E2')],
          expectedVersion: 0,
        ),
        throwsA(isA<VersionConflictError>()),
      );
    });

    test('VersionConflictError has expected and actual', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
      ]);
      try {
        await store.append(
          's1',
          [const NewEvent(streamId: 's1', eventType: 'E3')],
          expectedVersion: 1,
        );
        fail('should throw');
      } on VersionConflictError catch (e) {
        expect(e.expected, equals(1));
        expect(e.actual, equals(2));
        expect(e.toString(), contains('expected=1'));
      }
    });
  });

  group('loadFrom', () {
    test('loads events from specific version', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
        const NewEvent(streamId: 's1', eventType: 'E3'),
      ]);
      final events = await store.loadFrom('s1', 2);
      expect(events, hasLength(2));
      expect(events[0].version, equals(2));
      expect(events[1].version, equals(3));
    });
  });

  group('exists', () {
    test('returns false for nonexistent stream', () async {
      expect(await store.exists('nope'), isFalse);
    });

    test('returns true after append', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
      ]);
      expect(await store.exists('s1'), isTrue);
    });
  });

  group('currentVersion', () {
    test('returns 0 for nonexistent stream', () async {
      expect(await store.currentVersion('nope'), equals(0));
    });

    test('returns latest version', () async {
      await store.append('s1', [
        const NewEvent(streamId: 's1', eventType: 'E1'),
        const NewEvent(streamId: 's1', eventType: 'E2'),
      ]);
      expect(await store.currentVersion('s1'), equals(2));
    });
  });

  group('InMemorySnapshotStore', () {
    late InMemorySnapshotStore snapStore;

    setUp(() {
      snapStore = InMemorySnapshotStore();
    });

    test('returns null for nonexistent snapshot', () async {
      expect(await snapStore.loadSnapshot('nope'), isNull);
    });

    test('saves and loads snapshot', () async {
      final snap = Snapshot(
        streamId: 's1',
        version: 5,
        state: {'count': 42},
        createdAt: DateTime.now(),
      );
      await snapStore.saveSnapshot(snap);
      final loaded = await snapStore.loadSnapshot('s1');
      expect(loaded, isNotNull);
      expect(loaded!.version, equals(5));
      expect(loaded.streamId, equals('s1'));
    });

    test('overwrites existing snapshot', () async {
      await snapStore.saveSnapshot(Snapshot(
        streamId: 's1',
        version: 3,
        createdAt: DateTime.now(),
      ));
      await snapStore.saveSnapshot(Snapshot(
        streamId: 's1',
        version: 7,
        createdAt: DateTime.now(),
      ));
      final loaded = await snapStore.loadSnapshot('s1');
      expect(loaded!.version, equals(7));
    });
  });
}
