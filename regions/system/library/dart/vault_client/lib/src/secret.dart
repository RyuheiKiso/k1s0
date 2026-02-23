class Secret {
  final String path;
  final Map<String, String> data;
  final int version;
  final DateTime createdAt;

  const Secret({
    required this.path,
    required this.data,
    required this.version,
    required this.createdAt,
  });
}

class SecretRotatedEvent {
  final String path;
  final int version;

  const SecretRotatedEvent({
    required this.path,
    required this.version,
  });
}
