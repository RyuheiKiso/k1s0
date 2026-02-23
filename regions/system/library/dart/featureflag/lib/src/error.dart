class FeatureFlagNotFoundException implements Exception {
  final String flagKey;

  const FeatureFlagNotFoundException(this.flagKey);

  @override
  String toString() => 'FeatureFlagNotFoundException: flag "$flagKey" not found';
}
