import 'package:test/test.dart';
import 'package:k1s0_resiliency/resiliency.dart';

void main() {
  group('ResiliencyDecorator', () {
    test('should execute successfully', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy());
      final result = await decorator.execute(() async => 42);
      expect(result, equals(42));
    });

    test('should retry on failure', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        retry: RetryConfig(
          maxAttempts: 3,
          baseDelay: Duration(milliseconds: 10),
          maxDelay: Duration(milliseconds: 100),
        ),
      ));

      var counter = 0;
      final result = await decorator.execute(() async {
        counter++;
        if (counter < 3) throw Exception('fail');
        return 99;
      });

      expect(result, equals(99));
      expect(counter, equals(3));
    });

    test('should throw MaxRetriesExceededError', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        retry: RetryConfig(
          maxAttempts: 2,
          baseDelay: Duration(milliseconds: 1),
          maxDelay: Duration(milliseconds: 10),
        ),
      ));

      expect(
        () => decorator.execute(() async => throw Exception('always fail')),
        throwsA(isA<MaxRetriesExceededError>()),
      );
    });

    test('should timeout', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        timeout: Duration(milliseconds: 50),
      ));

      expect(
        () => decorator.execute(() async {
          await Future<void>.delayed(const Duration(seconds: 1));
          return 42;
        }),
        throwsA(isA<TimeoutError>()),
      );
    });

    test('should open circuit breaker after failures', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        circuitBreaker: CircuitBreakerConfig(
          failureThreshold: 3,
          recoveryTimeout: Duration(minutes: 1),
          halfOpenMaxCalls: 1,
        ),
      ));

      // Trip the circuit
      for (var i = 0; i < 3; i++) {
        try {
          await decorator.execute(() async => throw Exception('fail'));
        } catch (_) {}
      }

      // Next call should fail with circuit open
      expect(
        () => decorator.execute(() async => 42),
        throwsA(isA<CircuitBreakerOpenError>()),
      );
    });

    test('should reject when bulkhead is full', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        bulkhead: BulkheadConfig(
          maxConcurrentCalls: 1,
          maxWaitDuration: Duration(milliseconds: 50),
        ),
      ));

      // Occupy the single slot
      final longRunning = decorator.execute(() async {
        await Future<void>.delayed(const Duration(milliseconds: 500));
        return 1;
      });

      await Future<void>.delayed(const Duration(milliseconds: 10));

      expect(
        () => decorator.execute(() async => 2),
        throwsA(isA<BulkheadFullError>()),
      );

      await longRunning;
    });
  });

  group('withResiliency', () {
    test('should work as convenience function', () async {
      final result =
          await withResiliency(const ResiliencyPolicy(), () async => 42);
      expect(result, equals(42));
    });
  });
}
