import 'package:test/test.dart';
import 'package:k1s0_resiliency/resiliency.dart';

void main() {
  group('ResiliencyDecorator', () {
    test('正常実行: ポリシーなしで処理が成功すること', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy());
      final result = await decorator.execute(() async => 42);
      expect(result, equals(42));
    });

    test('リトライ: 失敗後に再試行して成功すること', () async {
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

    test('リトライ上限: 最大試行回数を超えると MaxRetriesExceededError が発生すること', () async {
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

    test('タイムアウト: 指定時間内に完了しない場合に TimeoutError が発生すること', () async {
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

    test('サーキットブレーカー: 連続失敗後にオープン状態へ遷移し CircuitBreakerOpenError が発生すること', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        circuitBreaker: CircuitBreakerConfig(
          failureThreshold: 3,
          recoveryTimeout: Duration(minutes: 1),
          halfOpenMaxCalls: 1,
        ),
      ));

      // しきい値分だけ失敗させてサーキットブレーカーをトリップさせる
      for (var i = 0; i < 3; i++) {
        try {
          await decorator.execute(() async => throw Exception('fail'));
        } catch (_) {}
      }

      // オープン状態では次の呼び出しが即座に拒否される
      expect(
        () => decorator.execute(() async => 42),
        throwsA(isA<CircuitBreakerOpenError>()),
      );
    });

    test('バルクヘッド: 同時実行数が上限に達すると BulkheadFullError が発生すること', () async {
      final decorator = ResiliencyDecorator(const ResiliencyPolicy(
        bulkhead: BulkheadConfig(
          maxConcurrentCalls: 1,
          maxWaitDuration: Duration(milliseconds: 50),
        ),
      ));

      // 唯一のスロットを長時間占有する
      final longRunning = decorator.execute(() async {
        await Future<void>.delayed(const Duration(milliseconds: 500));
        return 1;
      });

      await Future<void>.delayed(const Duration(milliseconds: 10));

      // スロットが埋まっているため次の呼び出しは拒否される
      expect(
        () => decorator.execute(() async => 2),
        throwsA(isA<BulkheadFullError>()),
      );

      await longRunning;
    });
  });

  group('withResiliency', () {
    test('便利関数: ポリシーを適用して正常実行できること', () async {
      final result =
          await withResiliency(const ResiliencyPolicy(), () async => 42);
      expect(result, equals(42));
    });
  });
}
