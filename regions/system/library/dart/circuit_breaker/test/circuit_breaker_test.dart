import 'package:test/test.dart';

import 'package:k1s0_circuit_breaker/circuit_breaker.dart';

void main() {
  late CircuitBreaker breaker;

  setUp(() {
    breaker = CircuitBreaker(const CircuitBreakerConfig(
      failureThreshold: 3,
      successThreshold: 2,
      timeout: Duration(milliseconds: 100),
    ));
  });

  group('initial state', () {
    test('starts closed', () {
      expect(breaker.state, equals(CircuitState.closed));
      expect(breaker.isOpen, isFalse);
    });
  });

  group('failure tracking', () {
    test('opens after threshold failures', () {
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.closed));
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.open));
    });

    test('resets failure count on success', () {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordSuccess();
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.closed));
    });
  });

  group('call', () {
    test('executes function when closed', () async {
      final result = await breaker.call(() async => 42);
      expect(result, equals(42));
    });

    test('throws when open', () async {
      for (var i = 0; i < 3; i++) {
        breaker.recordFailure();
      }
      expect(
        () => breaker.call(() async => 1),
        throwsA(isA<CircuitBreakerException>()),
      );
    });

    test('records failure on exception', () async {
      for (var i = 0; i < 3; i++) {
        try {
          await breaker.call(() async => throw Exception('fail'));
        } catch (_) {}
      }
      expect(breaker.isOpen, isTrue);
    });
  });

  group('half-open', () {
    test('transitions to half-open after timeout', () async {
      for (var i = 0; i < 3; i++) {
        breaker.recordFailure();
      }
      expect(breaker.state, equals(CircuitState.open));
      await Future<void>.delayed(const Duration(milliseconds: 150));
      expect(breaker.state, equals(CircuitState.halfOpen));
    });
  });
}
