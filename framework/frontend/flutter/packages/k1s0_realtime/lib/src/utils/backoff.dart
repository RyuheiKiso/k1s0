import 'dart:math';

import '../types/reconnect_config.dart';

final _random = Random();

/// バックオフ遅延を計算する
///
/// [attempt] 試行回数（0始まり）
/// [initialDelay] 初回遅延
/// [maxDelay] 最大遅延
/// [backoffType] バックオフ戦略
Duration calculateBackoff(
  int attempt,
  Duration initialDelay,
  Duration maxDelay,
  BackoffType backoffType,
) {
  int delayMs;

  if (backoffType == BackoffType.exponential) {
    delayMs = initialDelay.inMilliseconds * pow(2, attempt).toInt();
  } else {
    delayMs = initialDelay.inMilliseconds * (attempt + 1);
  }

  final cappedMs = min(delayMs, maxDelay.inMilliseconds);
  return Duration(milliseconds: cappedMs);
}

/// 遅延にランダムなジッターを追加する（±25%）
Duration addJitter(Duration delay) {
  final factor = 0.75 + _random.nextDouble() * 0.5; // 0.75 ~ 1.25
  final jitteredMs = (delay.inMilliseconds * factor).round();
  return Duration(milliseconds: jitteredMs);
}
