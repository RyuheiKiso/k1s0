class JobCompletedEvent {
  final String jobId;
  final String executionId;
  final DateTime completedAt;
  final String result;

  const JobCompletedEvent({
    required this.jobId,
    required this.executionId,
    required this.completedAt,
    required this.result,
  });
}
