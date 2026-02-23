class FlagVariant {
  final String name;
  final String value;
  final double weight;

  const FlagVariant({
    required this.name,
    required this.value,
    this.weight = 1.0,
  });
}

class FeatureFlag {
  final String id;
  final String flagKey;
  final String description;
  final bool enabled;
  final List<FlagVariant> variants;

  const FeatureFlag({
    required this.id,
    required this.flagKey,
    required this.description,
    required this.enabled,
    this.variants = const [],
  });
}
