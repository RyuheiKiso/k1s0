import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

void main() {
  group('InMemoryStateStore', () {
    late InMemoryStateStore store;

    setUp(() {
      store = InMemoryStateStore();
    });

    test('初期状態は uninitialized', () async {
      expect(await store.status(), ComponentStatus.uninitialized);
    });

    test('init 後は ready になる', () async {
      await store.init();
      expect(await store.status(), ComponentStatus.ready);
    });

    test('close 後は closed になりエントリがクリアされる', () async {
      await store.init();
      await store.set('k', Uint8List.fromList([1]));
      await store.close();
      expect(await store.status(), ComponentStatus.closed);
      expect(await store.get('k'), isNull);
    });

    test('デフォルト name は inmemory-statestore', () {
      expect(store.name, 'inmemory-statestore');
      expect(store.componentType, 'statestore');
    });

    test('コンストラクタで name を指定できる', () {
      final named = InMemoryStateStore(name: 'custom-store');
      expect(named.name, 'custom-store');
    });

    test('metadata は backend=memory を返す', () {
      expect(store.metadata(), {'backend': 'memory'});
    });

    test('set した値を get で取得できる', () async {
      await store.init();
      final etag = await store.set('key', Uint8List.fromList([10, 20, 30]));
      final entry = await store.get('key');

      expect(entry, isNotNull);
      expect(entry!.key, 'key');
      expect(entry.value, Uint8List.fromList([10, 20, 30]));
      expect(entry.etag, etag);
    });

    test('存在しないキーの get は null を返す', () async {
      await store.init();
      expect(await store.get('missing'), isNull);
    });

    test('set は毎回新しい ETag を返す', () async {
      await store.init();
      final etag1 = await store.set('k', Uint8List.fromList([1]));
      final etag2 = await store.set('k', Uint8List.fromList([2]));
      expect(etag1, isNot(equals(etag2)));
    });

    test('正しい ETag で set すると成功する', () async {
      await store.init();
      final etag = await store.set('k', Uint8List.fromList([1]));
      final newEtag = await store.set('k', Uint8List.fromList([2]), etag: etag);
      expect(newEtag, isNot(equals(etag)));

      final entry = await store.get('k');
      expect(entry!.value, Uint8List.fromList([2]));
    });

    test('古い ETag で set すると ETagMismatchError になる', () async {
      await store.init();
      await store.set('k', Uint8List.fromList([1]));
      expect(
        () => store.set('k', Uint8List.fromList([2]), etag: 'stale'),
        throwsA(isA<ETagMismatchError>()),
      );
    });

    test('存在しないキーに ETag 付きで set すると ETagMismatchError になる', () async {
      await store.init();
      expect(
        () => store.set('missing', Uint8List.fromList([1]), etag: 'any'),
        throwsA(isA<ETagMismatchError>()),
      );
    });

    test('delete でエントリを削除できる', () async {
      await store.init();
      final etag = await store.set('k', Uint8List.fromList([1]));
      await store.delete('k', etag: etag);
      expect(await store.get('k'), isNull);
    });

    test('存在しないキーの delete はエラーにならない', () async {
      await store.init();
      expect(() => store.delete('missing'), returnsNormally);
    });

    test('ETag なしで delete すると無条件削除される', () async {
      await store.init();
      await store.set('k', Uint8List.fromList([1]));
      await store.delete('k');
      expect(await store.get('k'), isNull);
    });

    test('古い ETag で delete すると ETagMismatchError になる', () async {
      await store.init();
      await store.set('k', Uint8List.fromList([1]));
      expect(
        () => store.delete('k', etag: 'stale'),
        throwsA(isA<ETagMismatchError>()),
      );
    });

    test('bulkGet で複数エントリを取得できる', () async {
      await store.init();
      await store.set('a', Uint8List.fromList([1]));
      await store.set('b', Uint8List.fromList([2]));

      final entries = await store.bulkGet(['a', 'b']);
      expect(entries, hasLength(2));
      expect(entries[0].value, Uint8List.fromList([1]));
      expect(entries[1].value, Uint8List.fromList([2]));
    });

    test('bulkGet は存在しないキーをスキップする', () async {
      await store.init();
      await store.set('a', Uint8List.fromList([1]));

      final entries = await store.bulkGet(['a', 'missing']);
      expect(entries, hasLength(1));
      expect(entries[0].key, 'a');
    });

    test('bulkSet で複数エントリを一括保存できる', () async {
      await store.init();
      final etags = await store.bulkSet([
        StateEntry(key: 'x', value: Uint8List.fromList([10]), etag: ''),
        StateEntry(key: 'y', value: Uint8List.fromList([20]), etag: ''),
      ]);
      expect(etags, hasLength(2));

      final x = await store.get('x');
      final y = await store.get('y');
      expect(x!.value, Uint8List.fromList([10]));
      expect(y!.value, Uint8List.fromList([20]));
    });
  });
}
