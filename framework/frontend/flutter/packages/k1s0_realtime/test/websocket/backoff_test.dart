import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/types/reconnect_config.dart';
import 'package:k1s0_realtime/src/utils/backoff.dart';

void main() {
  group('calculateBackoff', () {
    group('linear backoff', () {
      test('試行回数に比例して遅延が増加する', () {
        expect(
          calculateBackoff(0, const Duration(seconds: 1),
              const Duration(seconds: 30), BackoffType.linear),
          const Duration(seconds: 1),
        );
        expect(
          calculateBackoff(1, const Duration(seconds: 1),
              const Duration(seconds: 30), BackoffType.linear),
          const Duration(seconds: 2),
        );
        expect(
          calculateBackoff(2, const Duration(seconds: 1),
              const Duration(seconds: 30), BackoffType.linear),
          const Duration(seconds: 3),
        );
      });

      test('最大遅延を超えない', () {
        expect(
          calculateBackoff(100, const Duration(seconds: 1),
              const Duration(seconds: 30), BackoffType.linear),
          const Duration(seconds: 30),
        );
      });
    });

    group('exponential backoff', () {
      test('試行回数の指数関数で遅延が増加する', () {
        expect(
          calculateBackoff(0, const Duration(seconds: 1),
              const Duration(seconds: 60), BackoffType.exponential),
          const Duration(seconds: 1),
        );
        expect(
          calculateBackoff(1, const Duration(seconds: 1),
              const Duration(seconds: 60), BackoffType.exponential),
          const Duration(seconds: 2),
        );
        expect(
          calculateBackoff(2, const Duration(seconds: 1),
              const Duration(seconds: 60), BackoffType.exponential),
          const Duration(seconds: 4),
        );
        expect(
          calculateBackoff(3, const Duration(seconds: 1),
              const Duration(seconds: 60), BackoffType.exponential),
          const Duration(seconds: 8),
        );
      });

      test('最大遅延を超えない', () {
        expect(
          calculateBackoff(20, const Duration(seconds: 1),
              const Duration(seconds: 30), BackoffType.exponential),
          const Duration(seconds: 30),
        );
      });
    });
  });

  group('addJitter', () {
    test('元の値の 75%~125% の範囲内に収まる', () {
      const delay = Duration(seconds: 10);
      for (var i = 0; i < 100; i++) {
        final result = addJitter(delay);
        expect(result.inMilliseconds, greaterThanOrEqualTo(7500));
        expect(result.inMilliseconds, lessThanOrEqualTo(12500));
      }
    });
  });
}
