import 'package:test/test.dart';

import 'package:k1s0_health/health.dart';

class AlwaysHealthy implements HealthCheck {
  @override
  String get name => 'always-healthy';

  @override
  Future<void> check() async {}
}

class AlwaysUnhealthy implements HealthCheck {
  @override
  String get name => 'always-unhealthy';

  @override
  Future<void> check() async => throw Exception('down');
}

void main() {
  group('HealthChecker', () {
    test('returns healthy when no checks', () async {
      final checker = HealthChecker();
      final resp = await checker.runAll();
      expect(resp.status, equals(HealthStatus.healthy));
      expect(resp.checks, isEmpty);
    });

    test('returns healthy when all checks pass', () async {
      final checker = HealthChecker()..add(AlwaysHealthy());
      final resp = await checker.runAll();
      expect(resp.status, equals(HealthStatus.healthy));
      expect(resp.checks['always-healthy']?.status, equals(HealthStatus.healthy));
    });

    test('returns unhealthy when a check fails', () async {
      final checker = HealthChecker()
        ..add(AlwaysHealthy())
        ..add(AlwaysUnhealthy());
      final resp = await checker.runAll();
      expect(resp.status, equals(HealthStatus.unhealthy));
      expect(resp.checks['always-unhealthy']?.status, equals(HealthStatus.unhealthy));
    });

    test('includes timestamp', () async {
      final checker = HealthChecker();
      final resp = await checker.runAll();
      expect(resp.timestamp, isA<DateTime>());
    });
  });

  group('CheckResult', () {
    test('stores status and message', () {
      const result = CheckResult(status: HealthStatus.degraded, message: 'slow');
      expect(result.status, equals(HealthStatus.degraded));
      expect(result.message, equals('slow'));
    });
  });
}
