/// Strategy for merging configuration values
enum MergeStrategy {
  /// Replace the entire value with the override
  replace,

  /// Deep merge maps and lists
  deepMerge,

  /// Append lists, merge maps
  append,
}

/// Configuration merge utilities
class ConfigMerger {
  /// Deep merge two maps
  static Map<String, dynamic> deepMerge(
    Map<String, dynamic> base,
    Map<String, dynamic> override, {
    MergeStrategy listStrategy = MergeStrategy.replace,
  }) {
    final result = Map<String, dynamic>.from(base);

    for (final key in override.keys) {
      final baseValue = result[key];
      final overrideValue = override[key];

      if (baseValue is Map<String, dynamic> &&
          overrideValue is Map<String, dynamic>) {
        // Recursively merge maps
        result[key] = deepMerge(baseValue, overrideValue, listStrategy: listStrategy);
      } else if (baseValue is List && overrideValue is List) {
        // Handle lists based on strategy
        result[key] = _mergeList(baseValue, overrideValue, listStrategy);
      } else {
        // Replace the value
        result[key] = overrideValue;
      }
    }

    return result;
  }

  static List<dynamic> _mergeList(
    List<dynamic> base,
    List<dynamic> override,
    MergeStrategy strategy,
  ) {
    switch (strategy) {
      case MergeStrategy.replace:
        return override;
      case MergeStrategy.deepMerge:
        return override;
      case MergeStrategy.append:
        return [...base, ...override];
    }
  }

  /// Merge multiple configuration maps in order
  static Map<String, dynamic> mergeAll(
    List<Map<String, dynamic>> configs, {
    MergeStrategy listStrategy = MergeStrategy.replace,
  }) {
    if (configs.isEmpty) {
      return {};
    }

    var result = configs.first;
    for (var i = 1; i < configs.length; i++) {
      result = deepMerge(result, configs[i], listStrategy: listStrategy);
    }

    return result;
  }

  /// Merge environment-specific configuration
  ///
  /// Merges in order: default.yaml + {env}.yaml
  static Map<String, dynamic> mergeEnvironmentConfig(
    Map<String, dynamic>? defaultConfig,
    Map<String, dynamic>? envConfig,
  ) {
    final configs = <Map<String, dynamic>>[];

    if (defaultConfig != null) {
      configs.add(defaultConfig);
    }

    if (envConfig != null) {
      configs.add(envConfig);
    }

    return mergeAll(configs);
  }
}
