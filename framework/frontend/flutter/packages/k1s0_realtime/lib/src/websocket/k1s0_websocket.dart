import 'dart:async';
import 'dart:convert';

import '../types/connection_status.dart';
import '../types/heartbeat_config.dart';
import '../types/reconnect_config.dart';
import 'heartbeat_handler.dart';
import 'reconnect_handler.dart';
import 'websocket_client.dart';

/// k1s0 WebSocket クライアント
///
/// 自動再接続、ハートビート、認証トークン付与を備えた WebSocket クライアント。
class K1s0WebSocket {
  /// 接続先 URL
  final String url;

  /// 再接続設定
  final ReconnectConfig reconnectConfig;

  /// ハートビート設定
  final HeartbeatConfig heartbeatConfig;

  /// 認証トークン取得関数
  final String? Function()? getAuthToken;

  /// デシリアライズ関数
  final dynamic Function(String)? deserialize;

  /// シリアライズ関数
  final String Function(dynamic)? serialize;

  late final WebSocketClientInternal _client;
  late final ReconnectHandler _reconnect;
  late final HeartbeatHandler _heartbeat;
  final StreamController<dynamic> _messageController =
      StreamController<dynamic>.broadcast();
  bool _intentionalDisconnect = false;

  K1s0WebSocket({
    required this.url,
    this.reconnectConfig = const ReconnectConfig(),
    this.heartbeatConfig = const HeartbeatConfig(),
    this.getAuthToken,
    this.deserialize,
    this.serialize,
  }) {
    _client = WebSocketClientInternal();
    _reconnect = ReconnectHandler(config: reconnectConfig);
    _heartbeat = HeartbeatHandler(config: heartbeatConfig);
  }

  /// 接続状態のストリーム
  Stream<ConnectionStatus> get status => _client.statusStream;

  /// メッセージのストリーム
  Stream<dynamic> get messages => _messageController.stream;

  /// 現在の接続状態
  ConnectionStatus get currentStatus => _client.status;

  /// 再接続試行回数
  int get reconnectAttempt => _reconnect.attempt;

  /// WebSocket 接続を開始する
  Future<void> connect() async {
    _intentionalDisconnect = false;
    _reconnect.restart();

    var connectUrl = url;
    if (getAuthToken != null) {
      final token = getAuthToken!();
      if (token != null) {
        final separator = url.contains('?') ? '&' : '?';
        connectUrl = '$url${separator}token=${Uri.encodeComponent(token)}';
      }
    }

    // メッセージストリームの購読
    _client.messageStream.listen(
      (data) {
        if (_heartbeat.handleMessage(data)) return;

        if (deserialize != null && data is String) {
          _messageController.add(deserialize!(data));
        } else {
          _messageController.add(data);
        }
      },
      onError: (Object error) {
        _messageController.addError(error);
      },
    );

    // 状態変更の監視（再接続トリガー）
    _client.statusStream.listen((status) {
      if (status == ConnectionStatus.connected) {
        _reconnect.reset();
        _heartbeat.start(
          send: (data) => _client.send(data),
          onTimeout: () => _client.disconnect(),
        );
      } else if (status == ConnectionStatus.disconnected) {
        _heartbeat.stop();
        if (!_intentionalDisconnect) {
          _reconnect.schedule(() => connect());
        }
      }
    });

    await _client.connect(Uri.parse(connectUrl));
  }

  /// 接続を切断する
  Future<void> disconnect({int? code, String? reason}) async {
    _intentionalDisconnect = true;
    _reconnect.stop();
    _heartbeat.stop();
    await _client.disconnect(code: code, reason: reason);
  }

  /// メッセージを送信する
  void send(dynamic data) {
    if (serialize != null) {
      _client.send(serialize!(data));
    } else {
      _client.send(data);
    }
  }

  /// JSON メッセージを送信する
  void sendJson(Map<String, dynamic> data) {
    _client.send(jsonEncode(data));
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await disconnect();
    await _client.dispose();
    await _messageController.close();
  }
}
