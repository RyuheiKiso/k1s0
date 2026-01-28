import 'dart:async';

import '../types/heartbeat_config.dart';

/// ハートビート（ping/pong）を管理するクラス
class HeartbeatHandler {
  final HeartbeatConfig config;
  Timer? _pingTimer;
  Timer? _pongTimer;
  void Function(String)? _sendFn;
  void Function()? _onTimeout;

  HeartbeatHandler({this.config = const HeartbeatConfig()});

  /// ハートビートを開始する
  void start({
    required void Function(String data) send,
    required void Function() onTimeout,
  }) {
    if (!config.enabled) return;

    _sendFn = send;
    _onTimeout = onTimeout;
    stop();

    _pingTimer = Timer.periodic(config.interval, (_) {
      _sendPing();
    });
  }

  /// ハートビートを停止する
  void stop() {
    _pingTimer?.cancel();
    _pingTimer = null;
    _clearPongTimer();
  }

  /// 受信メッセージがハートビート応答かを判定する
  bool handleMessage(dynamic data) {
    if (config.validateResponse == null) {
      _clearPongTimer();
      return false;
    }

    if (config.validateResponse!(data)) {
      _clearPongTimer();
      return true;
    }

    return false;
  }

  void _sendPing() {
    if (_sendFn == null) return;

    try {
      _sendFn!(config.message);
    } catch (_) {
      return;
    }

    _pongTimer = Timer(config.timeout, () {
      _pongTimer = null;
      _onTimeout?.call();
    });
  }

  void _clearPongTimer() {
    _pongTimer?.cancel();
    _pongTimer = null;
  }
}
