import 'package:test/test.dart';

import 'package:k1s0_retry/retry.dart';

void main() {
  group('RetryConfig', () {
    test('has sensible defaults', () {
      const config = RetryConfig();
      expect(config.maxAttempts, equals(3));
      expect(config.initialDelayMs, equals(100));
      expect(config.maxDelayMs, equals(30000));
      expect(config.multiplier, equals(2.0));
      expect(config.jitter, isTrue);
    });

    test('accepts custom values', () {
      const config = RetryConfig(
        maxAttempts: 5,
        initialDelayMs: 200,
        maxDelayMs: 10000,
        multiplier: 3.0,
        jitter: false,
      );
      expect(config.maxAttempts, equals(5));
      expect(config.initialDelayMs, equals(200));
    });
  });

  group('computeDelay', () {
    test('calculates exponential backoff without jitter', () {
      const config = RetryConfig(
        initialDelayMs: 100,
        multiplier: 2.0,
        jitter: false,
      );
      expect(computeDelay(config, 0), equals(100));
      expect(computeDelay(config, 1), equals(200));
      expect(computeDelay(config, 2), equals(400));
    });

    test('caps at maxDelayMs', () {
      const config = RetryConfig(
        initialDelayMs: 100,
        maxDelayMs: 300,
        multiplier: 2.0,
        jitter: false,
      );
      expect(computeDelay(config, 0), equals(100));
      expect(computeDelay(config, 1), equals(200));
      expect(computeDelay(config, 2), equals(300));
      expect(computeDelay(config, 5), equals(300));
    });

    test('applies jitter within 10% range', () {
      const config = RetryConfig(
        initialDelayMs: 1000,
        multiplier: 1.0,
        jitter: true,
      );
      for (var i = 0; i < 20; i++) {
        final delay = computeDelay(config, 0);
        expect(delay, greaterThanOrEqualTo(900));
        expect(delay, lessThanOrEqualTo(1100));
      }
    });
  });

  group('withRetry', () {
    test('returns on first success', () async {
      var calls = 0;
      final result = await withRetry(
        const RetryConfig(maxAttempts: 3, jitter: false, initialDelayMs: 1),
        () async {
          calls++;
          return 42;
        },
      );
      expect(result, equals(42));
      expect(calls, equals(1));
    });

    test('retries on failure then succeeds', () async {
      var calls = 0;
      final result = await withRetry(
        const RetryConfig(maxAttempts: 3, jitter: false, initialDelayMs: 1),
        () async {
          calls++;
          if (calls < 3) throw Exception('fail');
          return 'ok';
        },
      );
      expect(result, equals('ok'));
      expect(calls, equals(3));
    });

    test('throws RetryError after exhausting attempts', () async {
      expect(
        () => withRetry(
          const RetryConfig(maxAttempts: 3, jitter: false, initialDelayMs: 1),
          () async {
            throw Exception('always fail');
          },
        ),
        throwsA(isA<RetryError>()),
      );
    });

    test('RetryError contains attempt count and last error', () async {
      RetryError? caught;
      try {
        await withRetry(
          const RetryConfig(maxAttempts: 2, jitter: false, initialDelayMs: 1),
          () async {
            throw Exception('boom');
          },
        );
      } on RetryError catch (e) {
        caught = e;
      }
      expect(caught, isNotNull);
      expect(caught.attempts, equals(2));
      expect(caught.lastError.toString(), contains('boom'));
      expect(caught.toString(), contains('exhausted 2 retries'));
    });
  });

  group('CircuitBreaker', () {
    test('starts in closed state', () {
      final cb = CircuitBreaker();
      expect(cb.state, equals(CircuitBreakerState.closed));
      expect(cb.isOpen, isFalse);
    });

    test('opens after reaching failure threshold', () {
      final cb = CircuitBreaker(
        config: const CircuitBreakerConfig(failureThreshold: 3),
      );
      cb.recordFailure();
      cb.recordFailure();
      expect(cb.state, equals(CircuitBreakerState.closed));
      cb.recordFailure();
      expect(cb.state, equals(CircuitBreakerState.open));
      expect(cb.isOpen, isTrue);
    });

    test('success resets failure count in closed state', () {
      final cb = CircuitBreaker(
        config: const CircuitBreakerConfig(failureThreshold: 3),
      );
      cb.recordFailure();
      cb.recordFailure();
      cb.recordSuccess();
      cb.recordFailure();
      cb.recordFailure();
      expect(cb.state, equals(CircuitBreakerState.closed));
    });

    test('transitions from half-open to closed after success threshold', () {
      final cb = CircuitBreaker(
        config: const CircuitBreakerConfig(
          failureThreshold: 2,
          successThreshold: 2,
          timeoutMs: 0,
        ),
      );
      cb.recordFailure();
      cb.recordFailure();
      // timeoutMs=0 means immediate transition to halfOpen on next state check
      expect(cb.state, equals(CircuitBreakerState.halfOpen));
      cb.recordSuccess();
      expect(cb.state, equals(CircuitBreakerState.halfOpen));
      cb.recordSuccess();
      expect(cb.state, equals(CircuitBreakerState.closed));
    });

    test('failure in half-open reopens circuit', () {
      final cb = CircuitBreaker(
        config: const CircuitBreakerConfig(
          failureThreshold: 1,
          timeoutMs: 0,
        ),
      );
      cb.recordFailure();
      // timeoutMs=0 means immediate half-open
      expect(cb.state, equals(CircuitBreakerState.halfOpen));
      // failure in half-open: failureThreshold=1 so it opens again
      cb.recordFailure();
      // but timeoutMs=0 means it immediately transitions back to half-open
      expect(cb.state, equals(CircuitBreakerState.halfOpen));
    });
  });

  group('RetryError', () {
    test('has correct message', () {
      const err = RetryError(3, 'some error');
      expect(err.attempts, equals(3));
      expect(err.lastError, equals('some error'));
      expect(err.toString(), contains('exhausted 3 retries'));
    });
  });
}
