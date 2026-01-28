import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/provider/offline_queue.dart';
import 'package:k1s0_realtime/src/provider/realtime_manager.dart';
import 'package:k1s0_realtime/src/types/connection_info.dart';
import 'package:k1s0_realtime/src/types/connection_status.dart';
import 'package:k1s0_realtime/src/types/offline_queue_config.dart';
import 'package:k1s0_realtime/src/types/realtime_config.dart';

void main() {
  group('統合テスト', () {
    test('Manager + OfflineQueue の連携が正しく動作する', () async {
      final manager = K1s0RealtimeManager(
        config: const RealtimeConfig(
          networkMonitorEnabled: false,
          offlineQueue: OfflineQueueConfig(
            enabled: true,
            maxSize: 10,
            persistToStorage: false,
          ),
        ),
      );

      // 接続を登録
      const info = ConnectionInfo(
        id: 'ws-chat',
        status: ConnectionStatus.disconnected,
      );
      manager.registerConnection('ws-chat', info);

      // オフライン中にメッセージをキュー
      manager.queue('ws-chat', {'type': 'message', 'text': 'hello'});
      manager.queue('ws-chat', {'type': 'message', 'text': 'world'});

      // キューの中身を確認
      final queued = manager.offlineQueue.getQueuedItems<Map<String, String>>('ws-chat');
      expect(queued, hasLength(2));

      // オンライン復帰（接続状態更新）
      manager.updateConnection('ws-chat', ConnectionStatus.connected);

      // キューをフラッシュ
      final items = manager.flush<Map<String, String>>('ws-chat');
      expect(items, hasLength(2));
      expect(items[0]['text'], equals('hello'));

      // フラッシュ後は空
      expect(
        manager.offlineQueue.getQueuedItems('ws-chat'),
        isEmpty,
      );

      await manager.dispose();
    });
  });
}
