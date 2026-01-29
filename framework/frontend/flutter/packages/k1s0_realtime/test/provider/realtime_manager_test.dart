import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/provider/realtime_manager.dart';
import 'package:k1s0_realtime/src/types/connection_info.dart';
import 'package:k1s0_realtime/src/types/connection_status.dart';
import 'package:k1s0_realtime/src/types/realtime_config.dart';
import 'package:k1s0_realtime/src/types/offline_queue_config.dart';

void main() {
  group('K1s0RealtimeManager', () {
    late K1s0RealtimeManager manager;

    setUp(() {
      manager = K1s0RealtimeManager(
        config: const RealtimeConfig(
          networkMonitorEnabled: false,
          offlineQueue: OfflineQueueConfig(
            enabled: true,
            persistToStorage: false,
          ),
        ),
      );
    });

    tearDown(() async {
      await manager.dispose();
    });

    test('接続を登録・解除できる', () async {
      const info = ConnectionInfo(
        id: 'ws-1',
        status: ConnectionStatus.connected,
      );

      manager.registerConnection('ws-1', info);
      expect(manager.currentConnections, hasLength(1));
      expect(manager.currentConnections['ws-1']?.status,
          equals(ConnectionStatus.connected));

      manager.unregisterConnection('ws-1');
      expect(manager.currentConnections, isEmpty);
    });

    test('接続状態を更新できる', () {
      const info = ConnectionInfo(
        id: 'ws-1',
        status: ConnectionStatus.connecting,
      );

      manager.registerConnection('ws-1', info);
      manager.updateConnection('ws-1', ConnectionStatus.connected);

      expect(manager.currentConnections['ws-1']?.status,
          equals(ConnectionStatus.connected));
    });

    test('接続変更がストリームで通知される', () async {
      const info = ConnectionInfo(
        id: 'ws-1',
        status: ConnectionStatus.connected,
      );

      final future = manager.connections.first;
      manager.registerConnection('ws-1', info);

      final result = await future;
      expect(result, hasLength(1));
      expect(result['ws-1']?.id, equals('ws-1'));
    });

    test('オフラインキューにメッセージを追加・フラッシュできる', () {
      manager.queue('conn1', {'type': 'chat', 'text': 'hello'});
      manager.queue('conn1', {'type': 'chat', 'text': 'world'});

      final items = manager.flush<Map<String, String>>('conn1');
      expect(items, hasLength(2));
    });
  });
}
