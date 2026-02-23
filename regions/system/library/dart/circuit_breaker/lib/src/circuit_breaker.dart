enum CircuitState { closed, open, halfOpen }

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

class CircuitBreakerException implements Exception {
  const CircuitBreakerException();

  @override
  String toString() => 'CircuitBreakerException: circuit is open';
}

class CircuitBreaker {
  final CircuitBreakerConfig config;

  int _failureCount = 0;
  int _successCount = 0;
  CircuitState _state = CircuitState.closed;
  DateTime? _openedAt;

  CircuitBreaker(this.config);

  CircuitState get state {
    if (_state == CircuitState.open) {
      final now = DateTime.now();
      if (_openedAt != null && now.difference(_openedAt!) >= config.timeout) {
        _state = CircuitState.halfOpen;
      }
    }
    return _state;
  }

  bool get isOpen => state == CircuitState.open;

  void recordSuccess() {
    if (_state == CircuitState.halfOpen) {
      _successCount++;
      if (_successCount >= config.successThreshold) {
        _state = CircuitState.closed;
        _failureCount = 0;
        _successCount = 0;
      }
    } else {
      _failureCount = 0;
    }
  }

  void recordFailure() {
    _failureCount++;
    _successCount = 0;
    if (_failureCount >= config.failureThreshold) {
      _state = CircuitState.open;
      _openedAt = DateTime.now();
    }
  }

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
