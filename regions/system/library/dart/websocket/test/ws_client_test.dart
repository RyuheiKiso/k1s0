import 'dart:typed_data';

import 'package:test/test.dart';
import 'package:k1s0_websocket/websocket.dart';

void main() {
  late InMemoryWsClient client;

  setUp(() {
    client = InMemoryWsClient();
  });

  group('WsMessage', () {
    test('creates text message', () {
      const msg = WsMessage(type: MessageType.text, payload: 'hello');
      expect(msg.textPayload, equals('hello'));
      expect(msg.type, equals(MessageType.text));
    });

    test('creates binary message', () {
      final data = Uint8List.fromList([1, 2, 3]);
      final msg = WsMessage(type: MessageType.binary, payload: data);
      expect(msg.binaryPayload, equals(data));
    });
  });

  group('WsConfig', () {
    test('defaults has expected values', () {
      final config = WsConfig.defaults;
      expect(config.url, equals('ws://localhost'));
      expect(config.reconnect, isTrue);
      expect(config.maxReconnectAttempts, equals(5));
      expect(config.reconnectDelay, equals(const Duration(seconds: 1)));
      expect(config.pingInterval, isNull);
    });

    test('custom config', () {
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
    test('has all values', () {
      expect(ConnectionState.values, hasLength(5));
    });
  });

  group('InMemoryWsClient', () {
    test('starts disconnected', () {
      expect(client.state, equals(ConnectionState.disconnected));
    });

    test('connect transitions to connected', () async {
      await client.connect();
      expect(client.state, equals(ConnectionState.connected));
    });

    test('disconnect transitions to disconnected', () async {
      await client.connect();
      await client.disconnect();
      expect(client.state, equals(ConnectionState.disconnected));
    });

    test('connect throws when already connected', () async {
      await client.connect();
      expect(() => client.connect(), throwsStateError);
    });

    test('disconnect throws when not connected', () async {
      expect(() => client.disconnect(), throwsStateError);
    });

    test('send stores messages', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.text, payload: 'test');
      await client.send(msg);
      expect(client.sentMessages, hasLength(1));
      expect(client.sentMessages.first.textPayload, equals('test'));
    });

    test('send throws when not connected', () async {
      const msg = WsMessage(type: MessageType.text, payload: 'test');
      expect(() => client.send(msg), throwsStateError);
    });

    test('receive returns injected messages', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.text, payload: 'incoming');
      client.injectMessage(msg);
      final received = await client.receive();
      expect(received.textPayload, equals('incoming'));
    });

    test('receive throws when not connected', () async {
      expect(() => client.receive(), throwsStateError);
    });

    test('receive throws when queue is empty', () async {
      await client.connect();
      expect(() => client.receive(), throwsStateError);
    });

    test('receive returns messages in order', () async {
      await client.connect();
      client.injectMessage(const WsMessage(type: MessageType.text, payload: 'first'));
      client.injectMessage(const WsMessage(type: MessageType.text, payload: 'second'));
      final first = await client.receive();
      final second = await client.receive();
      expect(first.textPayload, equals('first'));
      expect(second.textPayload, equals('second'));
    });

    test('send ping message', () async {
      await client.connect();
      const msg = WsMessage(type: MessageType.ping, payload: '');
      await client.send(msg);
      expect(client.sentMessages.first.type, equals(MessageType.ping));
    });
  });
}
