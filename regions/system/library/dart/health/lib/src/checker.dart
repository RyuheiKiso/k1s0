enum HealthStatus { healthy, degraded, unhealthy }

class CheckResult {
  final HealthStatus status;
  final String? message;

  const CheckResult({required this.status, this.message});
}

class HealthResponse {
  final HealthStatus status;
  final Map<String, CheckResult> checks;
  final DateTime timestamp;

  const HealthResponse({
    required this.status,
    required this.checks,
    required this.timestamp,
  });
}

abstract class HealthCheck {
  String get name;
  Future<void> check();
}

class HealthChecker {
  final List<HealthCheck> _checks = [];

  void add(HealthCheck check) => _checks.add(check);

  Future<HealthResponse> runAll() async {
    final results = <String, CheckResult>{};
    var overall = HealthStatus.healthy;

    for (final check in _checks) {
      try {
        await check.check();
        results[check.name] = const CheckResult(status: HealthStatus.healthy);
      } catch (e) {
        results[check.name] = CheckResult(
          status: HealthStatus.unhealthy,
          message: e.toString(),
        );
        overall = HealthStatus.unhealthy;
      }
    }

    return HealthResponse(
      status: overall,
      checks: results,
      timestamp: DateTime.now(),
    );
  }

  Future<HealthResponse> readyz() => runAll();

  Map<String, String> healthz() => {'status': 'ok'};
}
