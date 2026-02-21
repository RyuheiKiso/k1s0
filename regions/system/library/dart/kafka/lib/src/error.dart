class KafkaError implements Exception {
  final String message;
  final Object? cause;

  const KafkaError(this.message, {this.cause});

  @override
  String toString() => 'KafkaError: $message';
}
