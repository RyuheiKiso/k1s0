/// 耐障害性ライブラリで使用するエラー型の定義。
/// すべてのエラーは [ResiliencyError] を基底クラスとして持つ。

/// 耐障害性ライブラリ共通の基底エラークラス。
sealed class ResiliencyError implements Exception {
  final String message;
  const ResiliencyError(this.message);

  @override
  String toString() => 'ResiliencyError: $message';
}

/// 最大リトライ回数を超過した場合のエラー。
class MaxRetriesExceededError extends ResiliencyError {
  /// 試行した回数。
  final int attempts;

  /// 最後に発生したエラー。
  final Object? lastError;

  const MaxRetriesExceededError(this.attempts, this.lastError)
      : super('max retries exceeded');
}

/// サーキットブレーカーがオープン状態でリクエストを拒否した場合のエラー。
class CircuitBreakerOpenError extends ResiliencyError {
  /// サーキットブレーカーがクローズするまでの残り時間。
  final Duration remainingDuration;

  const CircuitBreakerOpenError(this.remainingDuration)
      : super('circuit breaker is open');
}

/// バルクヘッドの同時実行数が上限に達し、リクエストを受け付けられない場合のエラー。
class BulkheadFullError extends ResiliencyError {
  /// 設定されている最大同時実行数。
  final int maxConcurrent;

  const BulkheadFullError(this.maxConcurrent) : super('bulkhead full');
}

/// 処理がタイムアウト時間内に完了しなかった場合のエラー。
class TimeoutError extends ResiliencyError {
  /// 設定されていたタイムアウト時間。
  final Duration after;

  const TimeoutError(this.after) : super('operation timed out');
}
