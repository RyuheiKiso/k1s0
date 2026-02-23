enum CircuitBreakerState { closed, open, halfOpen }

class CircuitBreakerConfig {
  final int failureThreshold;
  final int successThreshold;
  final int timeoutMs;

  const CircuitBreakerConfig({
    this.failureThreshold = 5,
    this.successThreshold = 2,
    this.timeoutMs = 30000,
  });
}

class CircuitBreaker {
  final CircuitBreakerConfig config;
  CircuitBreakerState _state = CircuitBreakerState.closed;
  int _failureCount = 0;
  int _successCount = 0;
  int _openedAt = 0;

  CircuitBreaker({CircuitBreakerConfig? config})
      : config = config ?? const CircuitBreakerConfig();

  CircuitBreakerState get state {
    _checkTimeout();
    return _state;
  }

  bool get isOpen {
    _checkTimeout();
    return _state == CircuitBreakerState.open;
  }

  void recordSuccess() {
    _checkTimeout();
    if (_state == CircuitBreakerState.halfOpen) {
      _successCount++;
      if (_successCount >= config.successThreshold) {
        _state = CircuitBreakerState.closed;
        _failureCount = 0;
        _successCount = 0;
        _openedAt = 0;
      }
    } else if (_state == CircuitBreakerState.closed) {
      _failureCount = 0;
    }
  }

  void recordFailure() {
    _checkTimeout();
    _failureCount++;
    if (_failureCount >= config.failureThreshold) {
      _state = CircuitBreakerState.open;
      _openedAt = DateTime.now().millisecondsSinceEpoch;
      _failureCount = 0;
    }
  }

  void _checkTimeout() {
    if (_state == CircuitBreakerState.open && _openedAt > 0) {
      if (DateTime.now().millisecondsSinceEpoch - _openedAt >=
          config.timeoutMs) {
        _state = CircuitBreakerState.halfOpen;
        _successCount = 0;
      }
    }
  }
}
