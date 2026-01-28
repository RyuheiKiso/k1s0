import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/types/connection_status.dart';
import 'package:k1s0_realtime/src/types/sse_event.dart';

void main() {
  group('SSEEvent', () {
    test('基本的な SSEEvent が作成できる', () {
      const event = SSEEvent(
        eventType: 'message',
        data: '{"key":"value"}',
      );

      expect(event.eventType, equals('message'));
      expect(event.data, equals('{"key":"value"}'));
      expect(event.id, isNull);
      expect(event.retry, isNull);
    });

    test('parse で JSON データをデシリアライズできる', () {
      const event = SSEEvent(
        eventType: 'update',
        data: '{"name":"test","count":42}',
      );

      final result = event.parse((json) => json);
      expect(result['name'], equals('test'));
      expect(result['count'], equals(42));
    });

    test('toJson で JSON Map を取得できる', () {
      const event = SSEEvent(
        eventType: 'message',
        data: '{"hello":"world"}',
      );

      final json = event.toJson();
      expect(json['hello'], equals('world'));
    });

    test('toString が正しい形式を返す', () {
      const event = SSEEvent(
        id: '1',
        eventType: 'test',
        data: 'data',
      );

      expect(event.toString(), equals('SSEEvent(type: test, id: 1, data: data)'));
    });
  });

  group('ConnectionStatus', () {
    test('全ての状態値が利用可能', () {
      expect(ConnectionStatus.connecting.name, equals('connecting'));
      expect(ConnectionStatus.connected.name, equals('connected'));
      expect(ConnectionStatus.disconnecting.name, equals('disconnecting'));
      expect(ConnectionStatus.disconnected.name, equals('disconnected'));
    });
  });
}
