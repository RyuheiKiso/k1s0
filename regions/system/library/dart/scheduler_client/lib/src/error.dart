class SchedulerError implements Exception {
  final String message;
  final String code;

  const SchedulerError(this.message, this.code);

  @override
  String toString() => 'SchedulerError($code): $message';
}
