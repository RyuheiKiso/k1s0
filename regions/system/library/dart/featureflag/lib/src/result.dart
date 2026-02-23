class EvaluationResult {
  final bool enabled;
  final String? variant;
  final String reason;

  const EvaluationResult({
    required this.enabled,
    this.variant,
    required this.reason,
  });
}
