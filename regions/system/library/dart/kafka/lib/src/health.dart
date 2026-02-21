class KafkaHealthStatus {
  final bool healthy;
  final String message;
  final int brokerCount;

  const KafkaHealthStatus({
    required this.healthy,
    required this.message,
    required this.brokerCount,
  });
}

abstract class KafkaHealthChecker {
  Future<KafkaHealthStatus> healthCheck();
}

class NoOpKafkaHealthChecker implements KafkaHealthChecker {
  final KafkaHealthStatus status;
  final Exception? error;

  const NoOpKafkaHealthChecker({required this.status, this.error});

  @override
  Future<KafkaHealthStatus> healthCheck() async {
    if (error != null) throw error!;
    return status;
  }
}
