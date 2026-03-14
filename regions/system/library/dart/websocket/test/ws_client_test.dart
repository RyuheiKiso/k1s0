import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:k1s0_websocket/websocket.dart';

void main() {
  late InMemoryWsClient client;

  setUp(() {
    client = InMemoryWsClient();
  });

  group('WsMessage', () {
    test('テキストメッセージが生成されること', () {
      const msg = WsMessage(type: MessageType.text, payload: 'hello');
      expect(msg.textPayload, equals('hello'));
      expect(msg.type, equals(MessageType.text));
    });

    test('バイナリメッセージが生成されること', () {
      final data = Uint8List.fromList([1, 2, 3]);
      final msg = WsMessage(type: MessageType.binary, payload: data);
      expect(msg.binaryPayload, equals(data));
    });
  });

  group('WsConfig', () {
    test('デフォルト値が期待通りに設定されていること', () {
      final config = WsConfig.defaults;
      expect(config.url, equals('ws://localhost'));
      expect(config.reconnect, isTrue);
      expect(config.maxReconnectAttempts, equals(5));
      expect(config.reconnectDelay, equals(const Duration(seconds: 1)));
      expect(config.pingInterval, isNull);
    });

    test('カスタム設定が適用されること', () {
      const config = WsConfig(
        url: 'ws://example.com',
        reconnect: false,
        maxReconnectAttempts: 3,
        pingInterval: const Duration(seconds: 30),
      );
      expect(config.url, equals('ws://example.com'));
      expect(config.reconnect, isFalse);
      expect(config.maxReconnectAttempts, equals(3));
      expect(config.pingInterval, equals(const Duration(seconds: 30)));
    });
  });

  group('ConnectionState', () {
    test('全ての値が定義されていること', () {
      expect(ConnectionState.values, hasLength(5));
    });
  });

  group('InMemoryWsClient', () {
    test('初期状態が切断中であること', () {
      expect(client.state, equals(ConnectionState.disconnected));
    });

    test('connectで接続済み状態に遷移すること', () async {
      await client.connect();
      expect(client.state, equals(ConnectionState.connected));
    });

    test('disconnectで切断済み状態に遷移すること', () async {
      await client.connect();
      await client.disconnect();
      expect(client.state, equals(ConnectionState.disconnected));
    });

    test('接続済みの状態でconnectを呼ぶと例外がスローされること', () async {
      await client.connect();
      expect(() => client.connect(), throwsStateError);
    });

    test('未接続の状態でdisconnectを呼ぶと例外がスローされること', () async {
      expect(() => client.disconnect(), throwsStateError);
    });

    test('sendでメッセージが保存されること', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.text, payload: 'test');
      await client.send(msg);
      expect(client.sentMessages, hasLength(1));
      expect(client.sentMessages.first.textPayload, equals('test'));
    });

    test('未接続の状態でsendを呼ぶと例外がスローされること', () async {
      const msg = WsMessage(type: MessageType.text, payload: 'test');
      expect(() => client.send(msg), throwsStateError);
    });

    test('receiveで注入したメッセージが返されること', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.text, payload: 'incoming');
      client.injectMessage(msg);
      final received = await client.receive();
      expect(received.textPayload, equals('incoming'));
    });

    test('未接続の状態でreceiveを呼ぶと例外がスローされること', () async {
      expect(() => client.receive(), throwsStateError);
    });

    test('キューが空の状態でreceiveを呼ぶと例外がスローされること', () async {
      await client.connect();
      expect(() => client.receive(), throwsStateError);
    });

    test('receiveでメッセージが順序通りに返されること', () async {
      await client.connect();
      client.injectMessage(const WsMessage(type: MessageType.text, payload: 'first'));
      client.injectMessage(const WsMessage(type: MessageType.text, payload: 'second'));
      final first = await client.receive();
      final second = await client.receive();
      expect(first.textPayload, equals('first'));
      expect(second.textPayload, equals('second'));
    });

    test('pingメッセージが送信できること', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.ping, payload: '');
      await client.send(msg);
      expect(client.sentMessages.first.type, equals(MessageType.ping));
    });
  });
}
