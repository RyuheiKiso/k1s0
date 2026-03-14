import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

/// テスト用: 受信メッセージを記録するシンプルなハンドラー実装。
class _RecordingHandler implements MessageHandler {
  final List<Message> received = [];

  @override
  Future<void> handle(Message message) async {
    received.add(message);
  }
}

void main() {
  group('InMemoryPubSub', () {
    late InMemoryPubSub ps;

    setUp(() {
      ps = InMemoryPubSub();
    });

    test('初期状態は uninitialized', () async {
      expect(await ps.status(), ComponentStatus.uninitialized);
    });

    test('init 後は ready になる', () async {
      await ps.init();
      expect(await ps.status(), ComponentStatus.ready);
    });

    test('close 後は closed になる', () async {
      await ps.init();
      await ps.close();
      expect(await ps.status(), ComponentStatus.closed);
    });

    test('デフォルト name は inmemory-pubsub', () {
      expect(ps.name, 'inmemory-pubsub');
      expect(ps.componentType, 'pubsub');
    });

    test('コンストラクタで name を指定できる', () {
      final named = InMemoryPubSub(name: 'custom-ps');
      expect(named.name, 'custom-ps');
    });

    test('metadata は backend=memory を返す', () {
      expect(ps.metadata(), {'backend': 'memory'});
    });

    test('publish したメッセージが subscribe ハンドラーで受信できる', () async {
      await ps.init();
      final handler = _RecordingHandler();
      await ps.subscribe('orders', handler);

      await ps.publish('orders', Uint8List.fromList([1, 2, 3]));

      expect(handler.received, hasLength(1));
      expect(handler.received[0].topic, 'orders');
      expect(handler.received[0].data, Uint8List.fromList([1, 2, 3]));
    });

    test('メッセージにユニークな ID が付与される', () async {
      await ps.init();
      final handler = _RecordingHandler();
      await ps.subscribe('t', handler);

      await ps.publish('t', Uint8List.fromList([1]));
      await ps.publish('t', Uint8List.fromList([2]));

      expect(handler.received[0].id, isNotEmpty);
      expect(handler.received[1].id, isNotEmpty);
      expect(handler.received[0].id, isNot(equals(handler.received[1].id)));
    });

    test('メタデータ付きで publish できる', () async {
      await ps.init();
      final handler = _RecordingHandler();
      await ps.subscribe('t', handler);

      await ps.publish('t', Uint8List(0), metadata: {'key': 'value'});

      expect(handler.received[0].metadata['key'], 'value');
    });

    test('複数のサブスクライバーが同じトピックを受信できる', () async {
      await ps.init();
      final h1 = _RecordingHandler();
      final h2 = _RecordingHandler();
      await ps.subscribe('events', h1);
      await ps.subscribe('events', h2);

      await ps.publish('events', Uint8List.fromList([42]));

      expect(h1.received, hasLength(1));
      expect(h2.received, hasLength(1));
    });

    test('別トピックのメッセージは受信しない', () async {
      await ps.init();
      final handler = _RecordingHandler();
      await ps.subscribe('topic-a', handler);

      await ps.publish('topic-b', Uint8List.fromList([1]));

      expect(handler.received, isEmpty);
    });

    test('購読者がいないトピックへの publish はエラーにならない', () async {
      await ps.init();
      expect(() => ps.publish('empty', Uint8List(0)), returnsNormally);
    });

    test('unsubscribe 後はメッセージを受信しない', () async {
      await ps.init();
      final handler = _RecordingHandler();
      final subId = await ps.subscribe('t', handler);
      await ps.unsubscribe(subId);

      await ps.publish('t', Uint8List.fromList([1]));

      expect(handler.received, isEmpty);
    });
  });
}
