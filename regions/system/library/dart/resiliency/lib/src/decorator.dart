import 'dart:async';
import 'dart:math' as math;

import 'package:k1s0_circuit_breaker/circuit_breaker.dart' as standalone;

import 'bulkhead.dart';
import 'error.dart';
import 'policy.dart';

class ResiliencyDecorator {
  final ResiliencyPolicy policy;
  final Bulkhead? _bulkhead;
  final standalone.CircuitBreaker? _cb;

  ResiliencyDecorator(this.policy)
      : _bulkhead = policy.bulkhead != null
            ? Bulkhead.fromConfig(policy.bulkhead!)
            : null,
        _cb = policy.circuitBreaker != null
            ? standalone.CircuitBreaker(standalone.CircuitBreakerConfig(
                failureThreshold: policy.circuitBreaker!.failureThreshold,
                successThreshold: policy.circuitBreaker!.halfOpenMaxCalls,
                timeout: policy.circuitBreaker!.recoveryTimeout,
              ))
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
        _cb?.recordSuccess();
        return result;
      } on ResiliencyError {
        rethrow;
      } catch (e) {
        _cb?.recordFailure();
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
    if (_cb == null) return;

    if (_cb.isOpen) {
      final elapsed = _cb.state == standalone.CircuitState.open
          ? policy.circuitBreaker!.recoveryTimeout
          : Duration.zero;
      throw CircuitBreakerOpenError(elapsed);
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
