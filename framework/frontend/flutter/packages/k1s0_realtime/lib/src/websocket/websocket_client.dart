import 'dart:async';

import 'package:web_socket_channel/web_socket_channel.dart';

import '../types/connection_status.dart';

/// 低レベル WebSocket クライアント
///
/// [web_socket_channel] パッケージをラップし、Stream ベースの API を提供する。
class WebSocketClientInternal {
  WebSocketChannel? _channel;
  final StreamController<ConnectionStatus> _statusController =
      StreamController<ConnectionStatus>.broadcast();
  final StreamController<dynamic> _messageController =
      StreamController<dynamic>.broadcast();
  ConnectionStatus _status = ConnectionStatus.disconnected;

  /// 現在の接続状態
  ConnectionStatus get status => _status;

  /// 接続状態のストリーム
  Stream<ConnectionStatus> get statusStream => _statusController.stream;

  /// メッセージのストリーム
  Stream<dynamic> get messageStream => _messageController.stream;

  /// WebSocket 接続を開始する
  Future<void> connect(Uri uri, {Iterable<String>? protocols}) async {
    if (_status == ConnectionStatus.connecting ||
        _status == ConnectionStatus.connected) {
      return;
    }

    _setStatus(ConnectionStatus.connecting);

    try {
      _channel = WebSocketChannel.connect(uri, protocols: protocols);
      await _channel!.ready;
      _setStatus(ConnectionStatus.connected);

      _channel!.stream.listen(
        (data) {
          _messageController.add(data);
        },
        onError: (Object error) {
          _messageController.addError(error);
          _setStatus(ConnectionStatus.disconnected);
        },
        onDone: () {
          _setStatus(ConnectionStatus.disconnected);
        },
      );
    } catch (e) {
      _setStatus(ConnectionStatus.disconnected);
      rethrow;
    }
  }

  /// WebSocket 接続を切断する
  Future<void> disconnect({int? code, String? reason}) async {
    if (_channel == null) return;

    _setStatus(ConnectionStatus.disconnecting);
    await _channel!.sink.close(code, reason);
    _channel = null;
    _setStatus(ConnectionStatus.disconnected);
  }

  /// メッセージを送信する
  void send(dynamic data) {
    if (_status != ConnectionStatus.connected || _channel == null) {
      throw StateError('WebSocket is not connected');
    }
    _channel!.sink.add(data);
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await disconnect();
    await _statusController.close();
    await _messageController.close();
  }

  void _setStatus(ConnectionStatus newStatus) {
    _status = newStatus;
    _statusController.add(newStatus);
  }
}
