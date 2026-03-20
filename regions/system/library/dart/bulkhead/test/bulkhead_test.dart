import 'dart:async';

import 'package:test/test.dart';
import 'package:k1s0_bulkhead/bulkhead.dart';

void main() {
  late BulkheadConfig config;
  late Bulkhead bulkhead;

  setUp(() {
    config = const BulkheadConfig(
      maxConcurrentCalls: 2,
      maxWaitDuration: Duration(milliseconds: 50),
    );
    bulkhead = Bulkhead(config);
  });

  test('acquire と release が正常に動作すること', () async {
    await bulkhead.acquire();
    await bulkhead.acquire();
    bulkhead.release();
    bulkhead.release();
  });

  test('タイムアウト後に満杯の場合は拒否されること', () async {
    await bulkhead.acquire();
    await bulkhead.acquire();

    // acquire()はFutureを返す非同期メソッドのため、expectLaterで非同期例外を検証する
    await expectLater(
      bulkhead.acquire(),
      throwsA(isA<BulkheadFullException>()),
    );
  });

  test('スロット解放まで待機すること', () async {
    await bulkhead.acquire();
    await bulkhead.acquire();

    final acquired = Completer<void>();
    unawaited(
      bulkhead.acquire().then((_) {
        acquired.complete();
      }),
    );

    await Future<void>.delayed(const Duration(milliseconds: 10));
    expect(acquired.isCompleted, isFalse);

    bulkhead.release();
    await acquired.future;
    expect(acquired.isCompleted, isTrue);

    bulkhead.release();
    bulkhead.release();
  });

  test('call が正常に成功すること', () async {
    final result = await bulkhead.call(() async => 42);
    expect(result, equals(42));
  });

  test('満杯の場合に call が拒否されること', () async {
    await bulkhead.acquire();
    await bulkhead.acquire();

    // call()はFutureを返す非同期メソッドのため、expectLaterで非同期例外を検証する
    await expectLater(
      bulkhead.call(() async => 1),
      throwsA(isA<BulkheadFullException>()),
    );

    bulkhead.release();
    bulkhead.release();
  });

  test('同時アクセスが上限を超えないこと', () async {
    var running = 0;
    var maxRunning = 0;

    final futures = List.generate(5, (i) {
      return bulkhead.call(() async {
        running++;
        if (running > maxRunning) maxRunning = running;
        await Future<void>.delayed(const Duration(milliseconds: 10));
        running--;
        return i;
      });
    });

    final results = await Future.wait(futures);
    expect(results, equals([0, 1, 2, 3, 4]));
    expect(maxRunning, lessThanOrEqualTo(2));
  });
}
