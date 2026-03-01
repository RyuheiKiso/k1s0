import 'dart:async';
import 'dart:collection';

import 'ws_message.dart';
import 'connection_state.dart';

abstract class WsClient {
  Future<void> connect();
  Future<void> disconnect();
  Future<void> send(WsMessage message);
  Future<WsMessage> receive();
  ConnectionState get state;
}

class InMemoryWsClient implements WsClient {
  final _receiveQueue = Queue<WsMessage>();
  final _sentMessages = <WsMessage>[];
  ConnectionState _state = ConnectionState.disconnected;

  @override
  ConnectionState get state => _state;

  List<WsMessage> get sentMessages => List.unmodifiable(_sentMessages);

  void injectMessage(WsMessage msg) {
    _receiveQueue.add(msg);
  }

  @override
  Future<void> connect() async {
    if (_state == ConnectionState.connected) {
      throw StateError('Already connected');
    }
    _state = ConnectionState.connecting;
    _state = ConnectionState.connected;
  }

  @override
  Future<void> disconnect() async {
    if (_state == ConnectionState.disconnected) {
      throw StateError('Not connected');
    }
    _state = ConnectionState.closing;
    _state = ConnectionState.disconnected;
  }

  @override
  Future<void> send(WsMessage message) async {
    if (_state != ConnectionState.connected) {
      throw StateError('Cannot send message while $state');
    }
    _sentMessages.add(message);
  }

  @override
  Future<WsMessage> receive() async {
    if (_state != ConnectionState.connected) {
      throw StateError('Not connected');
    }
    if (_receiveQueue.isEmpty) {
      throw StateError('No messages in receive queue');
    }
    return _receiveQueue.removeFirst();
  }
}
