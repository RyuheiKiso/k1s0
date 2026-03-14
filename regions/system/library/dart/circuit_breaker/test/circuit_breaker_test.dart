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

  group('初期状態', () {
    test('クローズド状態で開始すること', () {
      expect(breaker.state, equals(CircuitState.closed));
      expect(breaker.isOpen, isFalse);
    });
  });

  group('失敗の追跡', () {
    test('閾値を超える失敗でオープン状態に遷移すること', () {
      breaker.recordFailure();
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.closed));
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.open));
    });

    test('成功時に失敗カウントをリセットすること', () {
      breaker.recordFailure();
      breaker.recordFailure();
      breaker.recordSuccess();
      breaker.recordFailure();
      expect(breaker.state, equals(CircuitState.closed));
    });
  });

  group('call', () {
    test('クローズド状態で関数を実行できること', () async {
      final result = await breaker.call(() async => 42);
      expect(result, equals(42));
    });

    test('オープン状態で例外をスローすること', () async {
      for (var i = 0; i < 3; i++) {
        breaker.recordFailure();
      }
      expect(
        () => breaker.call(() async => 1),
        throwsA(isA<CircuitBreakerException>()),
      );
    });

    test('例外発生時に失敗を記録すること', () async {
      for (var i = 0; i < 3; i++) {
        try {
          await breaker.call(() async => throw Exception('fail'));
        } catch (_) {}
      }
      expect(breaker.isOpen, isTrue);
    });
  });

  group('ハーフオープン', () {
    test('タイムアウト後にハーフオープン状態へ遷移すること', () async {
      for (var i = 0; i < 3; i++) {
        breaker.recordFailure();
      }
      expect(breaker.state, equals(CircuitState.open));
      await Future<void>.delayed(const Duration(milliseconds: 150));
      expect(breaker.state, equals(CircuitState.halfOpen));
    });
  });
}
