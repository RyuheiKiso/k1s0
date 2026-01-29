/// バックオフ戦略
enum BackoffType {
  /// 線形バックオフ
  linear,

  /// 指数バックオフ
  exponential,
}

/// 再接続設定
class ReconnectConfig {
  /// 再接続を有効にする
  final bool enabled;

  /// 最大再接続回数（null = 無限）
  final int? maxAttempts;

  /// バックオフ戦略
  final BackoffType backoffType;

  /// 初回遅延
  final Duration initialDelay;

  /// 最大遅延
  final Duration maxDelay;

  /// ランダム揺らぎの追加
  final bool jitter;

  const ReconnectConfig({
    this.enabled = true,
    this.maxAttempts = 10,
    this.backoffType = BackoffType.exponential,
    this.initialDelay = const Duration(seconds: 1),
    this.maxDelay = const Duration(seconds: 30),
    this.jitter = true,
  });
}
