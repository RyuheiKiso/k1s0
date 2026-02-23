import 'dart:async';
import 'dart:math' as math;

import 'bulkhead.dart';
import 'error.dart';
import 'policy.dart';

enum _CircuitState { closed, open, halfOpen }

class ResiliencyDecorator {
  final ResiliencyPolicy policy;
  final Bulkhead? _bulkhead;
  _CircuitState _cbState = _CircuitState.closed;
  int _cbFailureCount = 0;
  int _cbSuccessCount = 0;
  DateTime? _cbLastFailureTime;

  ResiliencyDecorator(this.policy)
      : _bulkhead = policy.bulkhead != null
            ? Bulkhead(
                maxConcurrent: policy.bulkhead!.maxConcurrentCalls,
                maxWait: policy.bulkhead!.maxWaitDuration,
              )
            : null;

  Future<T> execute<T>(Future<T> Function() fn) async {
    _checkCircuitBreaker();

    if (_bulkhead != null) {
      await _bulkhead.acquire();
    }

    try {
      return await _executeWithRetry(fn);
    } finally {
      _bulkhead?.release();
    }
  }

  Future<T> _executeWithRetry<T>(Future<T> Function() fn) async {
    final maxAttempts = policy.retry?.maxAttempts ?? 1;
    Object? lastError;

    for (var attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        final result = await _executeWithTimeout(fn);
        _recordSuccess();
        return result;
      } on ResiliencyError {
        rethrow;
      } catch (e) {
        _recordFailure();
        lastError = e;

        _checkCircuitBreaker();

        if (attempt + 1 < maxAttempts && policy.retry != null) {
          final delay = _calculateBackoff(
            attempt,
            policy.retry!.baseDelay,
            policy.retry!.maxDelay,
          );
          await Future<void>.delayed(delay);
        }
      }
    }

    throw MaxRetriesExceededError(maxAttempts, lastError);
  }

  Future<T> _executeWithTimeout<T>(Future<T> Function() fn) async {
    if (policy.timeout == null) {
      return fn();
    }

    try {
      return await fn().timeout(policy.timeout!);
    } on TimeoutException {
      throw TimeoutError(policy.timeout!);
    }
  }

  void _checkCircuitBreaker() {
    if (policy.circuitBreaker == null) return;

    final cfg = policy.circuitBreaker!;
    switch (_cbState) {
      case _CircuitState.closed:
        return;
      case _CircuitState.open:
        if (_cbLastFailureTime != null) {
          final elapsed = DateTime.now().difference(_cbLastFailureTime!);
          if (elapsed >= cfg.recoveryTimeout) {
            _cbState = _CircuitState.halfOpen;
            _cbSuccessCount = 0;
            return;
          }
          throw CircuitBreakerOpenError(cfg.recoveryTimeout - elapsed);
        }
      case _CircuitState.halfOpen:
        return;
    }
  }

  void _recordSuccess() {
    if (policy.circuitBreaker == null) return;

    if (_cbState == _CircuitState.halfOpen) {
      _cbSuccessCount++;
      if (_cbSuccessCount >= policy.circuitBreaker!.halfOpenMaxCalls) {
        _cbState = _CircuitState.closed;
        _cbFailureCount = 0;
      }
    } else if (_cbState == _CircuitState.closed) {
      _cbFailureCount = 0;
    }
  }

  void _recordFailure() {
    if (policy.circuitBreaker == null) return;

    _cbFailureCount++;
    if (_cbFailureCount >= policy.circuitBreaker!.failureThreshold) {
      _cbState = _CircuitState.open;
      _cbLastFailureTime = DateTime.now();
    }
  }

  Duration _calculateBackoff(
    int attempt,
    Duration baseDelay,
    Duration maxDelay,
  ) {
    final delayMs =
        baseDelay.inMilliseconds * math.pow(2, attempt).toInt();
    final cappedMs = math.min(delayMs, maxDelay.inMilliseconds);
    return Duration(milliseconds: cappedMs);
  }
}

Future<T> withResiliency<T>(
  ResiliencyPolicy policy,
  Future<T> Function() fn,
) {
  return ResiliencyDecorator(policy).execute(fn);
}
