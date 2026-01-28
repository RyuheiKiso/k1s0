import 'dart:async';

import '../types/reconnect_config.dart';
import '../utils/backoff.dart';

/// 再接続を管理するクラス
class ReconnectHandler {
  final ReconnectConfig config;
  int _attempt = 0;
  Timer? _timer;
  bool _stopped = false;

  ReconnectHandler({this.config = const ReconnectConfig()});

  /// 現在の試行回数
  int get attempt => _attempt;

  /// 再接続をスケジュールする
  ///
  /// [onReconnect] 再接続実行時のコールバック
  /// [onAttempt] 試行開始時のコールバック
  /// 戻り値: スケジュールされた場合 true
  bool schedule(
    void Function() onReconnect, {
    void Function(int attempt)? onAttempt,
  }) {
    if (!config.enabled || _stopped) return false;
    if (config.maxAttempts != null && _attempt >= config.maxAttempts!) {
      return false;
    }

    _attempt++;

    var delay = calculateBackoff(
      _attempt - 1,
      config.initialDelay,
      config.maxDelay,
      config.backoffType,
    );

    if (config.jitter) {
      delay = addJitter(delay);
    }

    onAttempt?.call(_attempt);

    _timer = Timer(delay, () {
      _timer = null;
      if (!_stopped) {
        onReconnect();
      }
    });

    return true;
  }

  /// 試行回数をリセットする
  void reset() {
    _attempt = 0;
    cancel();
  }

  /// スケジュール済みの再接続をキャンセルする
  void cancel() {
    _timer?.cancel();
    _timer = null;
  }

  /// 再接続を完全に停止する
  void stop() {
    _stopped = true;
    cancel();
  }

  /// 停止状態を解除する
  void restart() {
    _stopped = false;
    _attempt = 0;
  }
}
