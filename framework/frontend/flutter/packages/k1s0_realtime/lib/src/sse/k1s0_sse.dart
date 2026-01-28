import 'dart:async';

import '../types/connection_status.dart';
import '../types/reconnect_config.dart';
import '../types/sse_event.dart';
import '../websocket/reconnect_handler.dart';
import 'sse_client.dart';

/// k1s0 SSE クライアント
///
/// 自動再接続、Last-Event-ID 管理を備えた SSE クライアント。
class K1s0SSE {
  /// 接続先 URL
  final String url;

  /// カスタムヘッダー
  final Map<String, String>? headers;

  /// 再接続設定
  final ReconnectConfig reconnectConfig;

  late final SSEClientInternal _client;
  late final ReconnectHandler _reconnect;
  String? _lastEventId;
  bool _intentionalDisconnect = false;

  K1s0SSE({
    required this.url,
    this.headers,
    this.reconnectConfig = const ReconnectConfig(),
  }) {
    _client = SSEClientInternal();
    _reconnect = ReconnectHandler(config: reconnectConfig);
  }

  /// イベントストリーム
  Stream<SSEEvent> get events => _client.eventStream.map((event) {
        if (event.id != null) {
          _lastEventId = event.id;
        }
        return event;
      });

  /// 接続状態のストリーム
  Stream<ConnectionStatus> get status => _client.statusStream;

  /// 現在の接続状態
  ConnectionStatus get currentStatus => _client.status;

  /// 接続を開始する
  Future<void> connect() async {
    _intentionalDisconnect = false;
    _reconnect.restart();

    _client.statusStream.listen((status) {
      if (status == ConnectionStatus.connected) {
        _reconnect.reset();
      } else if (status == ConnectionStatus.disconnected &&
          !_intentionalDisconnect) {
        _reconnect.schedule(() => _doConnect());
      }
    });

    await _doConnect();
  }

  /// 接続を切断する
  Future<void> disconnect() async {
    _intentionalDisconnect = true;
    _reconnect.stop();
    await _client.disconnect();
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await disconnect();
    await _client.dispose();
  }

  Future<void> _doConnect() async {
    await _client.connect(
      Uri.parse(url),
      headers: headers,
      lastEventId: _lastEventId,
    );
  }
}
