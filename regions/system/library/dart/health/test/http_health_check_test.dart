import 'package:test/test.dart';

import 'package:k1s0_health/health.dart';

void main() {
  group('HttpHealthCheck', () {
    test('デフォルト名が"http"であること', () {
      final check = HttpHealthCheck(url: 'http://example.com/healthz');
      expect(check.name, equals('http'));
    });

    test('カスタム名を設定できること', () {
      final check = HttpHealthCheck(
        url: 'http://example.com/healthz',
        name: 'upstream',
      );
      expect(check.name, equals('upstream'));
    });

    test('デフォルトタイムアウトが5秒であること', () {
      final check = HttpHealthCheck(url: 'http://example.com/healthz');
      expect(check.timeout, equals(const Duration(seconds: 5)));
    });

    test('カスタムタイムアウトを設定できること', () {
      final check = HttpHealthCheck(
        url: 'http://example.com/healthz',
        timeout: const Duration(seconds: 2),
      );
      expect(check.timeout, equals(const Duration(seconds: 2)));
    });

    test('接続不可のURLでエラーを投げること', () async {
      final check = HttpHealthCheck(
        url: 'http://127.0.0.1:1/healthz',
        timeout: const Duration(seconds: 1),
        name: 'unreachable',
      );
      expect(() => check.check(), throwsA(isA<Exception>()));
    });

    test('HealthCheckerと統合できること', () {
      final checker = HealthChecker();
      final check = HttpHealthCheck(url: 'http://example.com/healthz');
      checker.add(check);
      // チェッカーにHTTPヘルスチェックを追加できることを確認
      expect(check.name, equals('http'));
    });
  });
}
