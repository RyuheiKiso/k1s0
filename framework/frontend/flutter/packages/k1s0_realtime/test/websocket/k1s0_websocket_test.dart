import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/types/connection_status.dart';
import 'package:k1s0_realtime/src/types/reconnect_config.dart';
import 'package:k1s0_realtime/src/websocket/reconnect_handler.dart';

void main() {
  group('ReconnectHandler', () {
    test('有効時に再接続がスケジュールされる', () {
      final handler = ReconnectHandler(
        config: const ReconnectConfig(
          enabled: true,
          maxAttempts: 3,
          initialDelay: Duration(milliseconds: 100),
        ),
      );

      var called = false;
      final result = handler.schedule(() => called = true);

      expect(result, isTrue);
      expect(handler.attempt, equals(1));
    });

    test('無効時に再接続がスケジュールされない', () {
      final handler = ReconnectHandler(
        config: const ReconnectConfig(enabled: false),
      );

      final result = handler.schedule(() {});
      expect(result, isFalse);
    });

    test('最大試行回数に達するとスケジュールされない', () {
      final handler = ReconnectHandler(
        config: const ReconnectConfig(
          enabled: true,
          maxAttempts: 2,
          initialDelay: Duration(milliseconds: 10),
        ),
      );

      handler.schedule(() {});
      handler.schedule(() {});
      final result = handler.schedule(() {});

      expect(result, isFalse);
      expect(handler.attempt, equals(2));
    });

    test('reset で試行回数がリセットされる', () {
      final handler = ReconnectHandler(
        config: const ReconnectConfig(
          enabled: true,
          maxAttempts: 5,
          initialDelay: Duration(milliseconds: 10),
        ),
      );

      handler.schedule(() {});
      handler.schedule(() {});
      expect(handler.attempt, equals(2));

      handler.reset();
      expect(handler.attempt, equals(0));
    });

    test('stop で再接続が停止される', () {
      final handler = ReconnectHandler(
        config: const ReconnectConfig(
          enabled: true,
          initialDelay: Duration(milliseconds: 10),
        ),
      );

      handler.stop();
      final result = handler.schedule(() {});
      expect(result, isFalse);
    });
  });

  group('ConnectionStatus', () {
    test('全状態が定義されている', () {
      expect(ConnectionStatus.values.length, equals(4));
      expect(ConnectionStatus.values, contains(ConnectionStatus.connecting));
      expect(ConnectionStatus.values, contains(ConnectionStatus.connected));
      expect(ConnectionStatus.values, contains(ConnectionStatus.disconnecting));
      expect(ConnectionStatus.values, contains(ConnectionStatus.disconnected));
    });
  });
}
