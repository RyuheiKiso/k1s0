/// 耐障害性ポリシーの設定クラス群。
/// [ResiliencyPolicy] で各ポリシーをまとめて管理する。

/// リトライ動作の設定。
class RetryConfig {
  /// 最大試行回数（初回実行を含む）。
  final int maxAttempts;

  /// バックオフの基準遅延時間。
  final Duration baseDelay;

  /// バックオフの上限遅延時間。
  final Duration maxDelay;

  /// ジッター（ランダムな揺らぎ）を有効にするかどうか。
  final bool jitter;

  const RetryConfig({
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 100),
    this.maxDelay = const Duration(seconds: 5),
    this.jitter = true,
  });
}

/// サーキットブレーカー動作の設定。
class CircuitBreakerConfig {
  /// オープン状態に遷移するまでの連続失敗回数のしきい値。
  final int failureThreshold;

  /// オープン状態からハーフオープン状態に遷移するまでの待機時間。
  final Duration recoveryTimeout;

  /// ハーフオープン状態で許容する最大呼び出し回数。
  final int halfOpenMaxCalls;

  const CircuitBreakerConfig({
    this.failureThreshold = 5,
    this.recoveryTimeout = const Duration(seconds: 30),
    this.halfOpenMaxCalls = 2,
  });
}

/// バルクヘッド（同時実行数制限）の設定。
class BulkheadConfig {
  /// 許容する最大同時実行数。
  final int maxConcurrentCalls;

  /// スロット空き待ちの最大待機時間。これを超えると [BulkheadFullError] が発生する。
  final Duration maxWaitDuration;

  const BulkheadConfig({
    this.maxConcurrentCalls = 20,
    this.maxWaitDuration = const Duration(milliseconds: 500),
  });
}

/// 複数の耐障害性ポリシーをまとめたポリシー定義。
/// 各フィールドが null の場合、対応するポリシーは無効になる。
class ResiliencyPolicy {
  /// リトライポリシー（null の場合はリトライなし）。
  final RetryConfig? retry;

  /// サーキットブレーカーポリシー（null の場合は無効）。
  final CircuitBreakerConfig? circuitBreaker;

  /// バルクヘッドポリシー（null の場合は無効）。
  final BulkheadConfig? bulkhead;

  /// タイムアウト時間（null の場合は無制限）。
  final Duration? timeout;

  const ResiliencyPolicy({
    this.retry,
    this.circuitBreaker,
    this.bulkhead,
    this.timeout,
  });
}
