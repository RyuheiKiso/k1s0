import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:yaml/yaml.dart';

import 'config_types.dart';
import 'merge_strategy.dart';

/// Configuration load options
class ConfigLoadOptions {
  /// Creates configuration load options
  const ConfigLoadOptions({
    this.configDir = 'assets/config',
    this.defaultFileName = 'default.yaml',
    this.envFilePattern = '{env}.yaml',
    this.allowMissingDefault = true,
    this.allowMissingEnv = true,
  });

  /// Directory containing configuration files
  final String configDir;

  /// Default configuration file name
  final String defaultFileName;

  /// Environment-specific configuration file name pattern
  /// {env} will be replaced with the environment name
  final String envFilePattern;

  /// Whether to allow missing default configuration
  final bool allowMissingDefault;

  /// Whether to allow missing environment configuration
  final bool allowMissingEnv;
}

/// Configuration loader for loading and parsing YAML configuration files
class ConfigLoader {
  /// Creates a configuration loader
  ConfigLoader({
    this.options = const ConfigLoadOptions(),
  });

  /// Load options
  final ConfigLoadOptions options;

  /// Load configuration from assets
  Future<ConfigLoadResult> load(Environment env) async {
    try {
      final defaultConfig = await _loadYamlFile(
        '${options.configDir}/${options.defaultFileName}',
        allowMissing: options.allowMissingDefault,
      );

      final envFileName = options.envFilePattern.replaceAll('{env}', env.value);
      final envConfig = await _loadYamlFile(
        '${options.configDir}/$envFileName',
        allowMissing: options.allowMissingEnv,
      );

      // Merge configurations
      final merged = ConfigMerger.mergeEnvironmentConfig(
        defaultConfig,
        envConfig,
      );

      // Add environment to merged config
      merged['env'] = env.value;

      // Parse to AppConfig
      final appConfig = AppConfig.fromJson(merged);

      return ConfigLoadResult.success(config: appConfig);
    } catch (e, stackTrace) {
      return ConfigLoadResult.failure(
        message: 'Failed to load configuration: $e',
        error: e,
        stackTrace: stackTrace,
      );
    }
  }

  /// Load configuration from a specific path
  Future<ConfigLoadResult> loadFromPath(String path) async {
    try {
      final config = await _loadYamlFile(path, allowMissing: false);
      if (config == null) {
        return const ConfigLoadResult.failure(
          message: 'Configuration file not found',
        );
      }

      final appConfig = AppConfig.fromJson(config);
      return ConfigLoadResult.success(config: appConfig);
    } catch (e, stackTrace) {
      return ConfigLoadResult.failure(
        message: 'Failed to load configuration: $e',
        error: e,
        stackTrace: stackTrace,
      );
    }
  }

  /// Load and parse a YAML file from assets
  Future<Map<String, dynamic>?> _loadYamlFile(
    String path, {
    bool allowMissing = false,
  }) async {
    try {
      final content = await rootBundle.loadString(path);
      final yaml = loadYaml(content);

      if (yaml == null) {
        return null;
      }

      return _yamlToMap(yaml);
    } on FlutterError catch (_) {
      if (allowMissing) {
        return null;
      }
      rethrow;
    }
  }

  /// Convert YamlMap to Map<String, dynamic>
  Map<String, dynamic> _yamlToMap(dynamic yaml) {
    if (yaml is YamlMap) {
      return yaml.map(
        (key, value) => MapEntry(key.toString(), _yamlToMap(value)),
      );
    } else if (yaml is YamlList) {
      return {'_list': yaml.map(_yamlToMap).toList()};
    } else if (yaml is Map) {
      return yaml.map(
        (key, value) => MapEntry(key.toString(), _yamlToMap(value)),
      );
    } else if (yaml is List) {
      return {'_list': yaml.map(_yamlToMap).toList()};
    }
    return yaml as Map<String, dynamic>? ?? {};
  }
}

/// Configuration loader factory
class ConfigLoaderFactory {
  /// Creates a default configuration loader
  static ConfigLoader create({
    ConfigLoadOptions options = const ConfigLoadOptions(),
  }) {
    return ConfigLoader(options: options);
  }

  /// Creates a configuration loader for testing
  static TestConfigLoader createForTest(Map<String, dynamic> config) {
    return TestConfigLoader(config);
  }
}

/// Test configuration loader for unit testing
class TestConfigLoader extends ConfigLoader {
  /// Creates a test configuration loader
  TestConfigLoader(this.testConfig);

  /// Test configuration
  final Map<String, dynamic> testConfig;

  @override
  Future<ConfigLoadResult> load(Environment env) async {
    try {
      final config = Map<String, dynamic>.from(testConfig);
      config['env'] = env.value;
      final appConfig = AppConfig.fromJson(config);
      return ConfigLoadResult.success(config: appConfig);
    } catch (e, stackTrace) {
      return ConfigLoadResult.failure(
        message: 'Failed to parse test configuration: $e',
        error: e,
        stackTrace: stackTrace,
      );
    }
  }
}
