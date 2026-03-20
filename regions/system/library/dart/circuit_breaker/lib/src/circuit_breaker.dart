/// サーキットブレーカーの状態を表す列挙型。
enum CircuitState { closed, open, halfOpen }

/// サーキットブレーカーの設定。
class CircuitBreakerConfig {
  final int failureThreshold;
  final int successThreshold;
  final Duration timeout;

  const CircuitBreakerConfig({
    required this.failureThreshold,
    required this.successThreshold,
    required this.timeout,
  });
}

/// サーキットブレーカーが Open 状態のときにスローされる例外。
class CircuitBreakerException implements Exception {
  const CircuitBreakerException();

  @override
  String toString() => 'CircuitBreakerException: circuit is open';
}

/// サーキットブレーカーのメトリクススナップショット。
class CircuitBreakerMetrics {
  /// 成功回数の累計。
  final int successCount;

  /// 失敗回数の累計。
  final int failureCount;

  /// 現在の状態文字列（"Closed" / "Open" / "HalfOpen"）。
  final String state;

  const CircuitBreakerMetrics({
    required this.successCount,
    required this.failureCount,
    required this.state,
  });
}

/// CircuitState を表示用文字列に変換する。
String _stateToString(CircuitState s) {
  switch (s) {
    case CircuitState.open:
      return 'Open';
    case CircuitState.halfOpen:
      return 'HalfOpen';
    case CircuitState.closed:
      return 'Closed';
  }
}

/// サーキットブレーカー本体。
class CircuitBreaker {
  final CircuitBreakerConfig config;

  int _failureCount = 0;
  int _successCount = 0;
  CircuitState _state = CircuitState.closed;
  DateTime? _openedAt;

  // メトリクス累計カウンタ
  int _metricsSuccessCount = 0;
  int _metricsFailureCount = 0;

  CircuitBreaker(this.config);

  /// 現在の状態を返す。Open → HalfOpen のタイムアウト遷移も行う。
  CircuitState get state {
    if (_state == CircuitState.open) {
      final now = DateTime.now();
      if (_openedAt != null && now.difference(_openedAt!) >= config.timeout) {
        _state = CircuitState.halfOpen;
      }
    }
    return _state;
  }

  /// Open 状態かどうかを返す。
  bool get isOpen => state == CircuitState.open;

  /// 成功を記録する。Closed 状態では失敗カウントをリセットする。
  void recordSuccess() {
    _metricsSuccessCount++;
    if (_state == CircuitState.halfOpen) {
      _successCount++;
      if (_successCount >= config.successThreshold) {
        _state = CircuitState.closed;
        _failureCount = 0;
        _successCount = 0;
      }
    } else {
      // Closed 状態では成功時に失敗カウントをリセットする
      _failureCount = 0;
    }
  }

  /// 失敗を記録する。閾値超過または HalfOpen 状態で Open へ遷移する。
  void recordFailure() {
    _metricsFailureCount++;
    if (_state == CircuitState.halfOpen) {
      // HalfOpen 状態での失敗は即座に Open へ再遷移する
      _state = CircuitState.open;
      _openedAt = DateTime.now();
      _failureCount = 0;
      _successCount = 0;
      return;
    }
    _failureCount++;
    _successCount = 0;
    if (_failureCount >= config.failureThreshold) {
      _state = CircuitState.open;
      _openedAt = DateTime.now();
    }
  }

  /// 現在のメトリクススナップショットを返す。
  CircuitBreakerMetrics metrics() {
    return CircuitBreakerMetrics(
      successCount: _metricsSuccessCount,
      failureCount: _metricsFailureCount,
      state: _stateToString(_state),
    );
  }

  /// 関数を実行する。Open 状態の場合は CircuitBreakerException をスローする。
  Future<T> call<T>(Future<T> Function() fn) async {
    if (isOpen) {
      throw const CircuitBreakerException();
    }

    try {
      final result = await fn();
      recordSuccess();
      return result;
    } catch (e) {
      if (e is CircuitBreakerException) rethrow;
      recordFailure();
      rethrow;
    }
  }
}
