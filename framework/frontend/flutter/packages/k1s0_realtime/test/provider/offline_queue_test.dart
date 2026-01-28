import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/provider/offline_queue.dart';
import 'package:k1s0_realtime/src/types/offline_queue_config.dart';

void main() {
  group('OfflineQueue', () {
    late OfflineQueue queue;

    setUp(() {
      queue = OfflineQueue(
        config: const OfflineQueueConfig(
          enabled: true,
          maxSize: 5,
          persistToStorage: false,
        ),
      );
    });

    test('アイテムをキューに追加して取得できる', () {
      queue.queue('conn1', 'message1');
      queue.queue('conn1', 'message2');

      final items = queue.getQueuedItems<String>('conn1');
      expect(items, hasLength(2));
      expect(items[0], equals('message1'));
      expect(items[1], equals('message2'));
    });

    test('flush で全アイテムを取得してキューがクリアされる', () {
      queue.queue('conn1', 'msg1');
      queue.queue('conn1', 'msg2');

      final items = queue.flush<String>('conn1');
      expect(items, hasLength(2));
      expect(items[0], equals('msg1'));
      expect(items[1], equals('msg2'));

      // flush 後は空
      expect(queue.getQueuedItems('conn1'), isEmpty);
    });

    test('maxSize を超えると最古のアイテムが削除される', () {
      for (var i = 0; i < 7; i++) {
        queue.queue('conn1', 'msg$i');
      }

      final items = queue.getQueuedItems<String>('conn1');
      expect(items, hasLength(5));
      expect(items[0], equals('msg2')); // msg0, msg1 が削除されている
    });

    test('clearQueue でキューがクリアされる', () {
      queue.queue('conn1', 'msg1');
      queue.clearQueue('conn1');

      expect(queue.getQueuedItems('conn1'), isEmpty);
    });

    test('clearAll で全キューがクリアされる', () {
      queue.queue('conn1', 'msg1');
      queue.queue('conn2', 'msg2');

      queue.clearAll();

      expect(queue.getQueuedItems('conn1'), isEmpty);
      expect(queue.getQueuedItems('conn2'), isEmpty);
    });

    test('無効時はキューに追加されない', () {
      final disabledQueue = OfflineQueue(
        config: const OfflineQueueConfig(
          enabled: false,
          persistToStorage: false,
        ),
      );

      disabledQueue.queue('conn1', 'msg1');
      expect(disabledQueue.getQueuedItems('conn1'), isEmpty);
    });

    test('接続 ID ごとに独立したキューを管理する', () {
      queue.queue('conn1', 'a');
      queue.queue('conn2', 'b');

      expect(queue.getQueuedItems<String>('conn1'), equals(['a']));
      expect(queue.getQueuedItems<String>('conn2'), equals(['b']));
    });
  });
}
