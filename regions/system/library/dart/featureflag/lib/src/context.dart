class EvaluationContext {
  final String? userId;
  final String? tenantId;
  final Map<String, String> attributes;

  const EvaluationContext({
    this.userId,
    this.tenantId,
    this.attributes = const {},
  });
}
