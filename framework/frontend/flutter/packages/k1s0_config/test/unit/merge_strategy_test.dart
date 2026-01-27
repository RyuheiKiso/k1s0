import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/merge_strategy.dart';

void main() {
  group('deepMergeConfig', () {
    test('should merge simple maps', () {
      final base = {'a': 1, 'b': 2};
      final override = {'b': 3, 'c': 4};

      final result = deepMergeConfig(base, override);

      expect(result, {'a': 1, 'b': 3, 'c': 4});
    });

    test('should deep merge nested maps', () {
      final base = {
        'api': {
          'baseUrl': 'https://api.example.com',
          'timeout': 30000,
        },
        'logging': {'level': 'debug'},
      };
      final override = {
        'api': {
          'timeout': 60000,
          'retryCount': 3,
        },
      };

      final result = deepMergeConfig(base, override);

      expect(result, {
        'api': {
          'baseUrl': 'https://api.example.com',
          'timeout': 60000,
          'retryCount': 3,
        },
        'logging': {'level': 'debug'},
      });
    });

    test('should replace lists by default', () {
      final base = {
        'features': ['feature1', 'feature2'],
      };
      final override = {
        'features': ['feature3'],
      };

      final result = deepMergeConfig(
        base,
        override,
        listStrategy: MergeStrategy.replace,
      );

      expect(result, {
        'features': ['feature3'],
      });
    });

    test('should append lists when strategy is append', () {
      final base = {
        'features': ['feature1', 'feature2'],
      };
      final override = {
        'features': ['feature3'],
      };

      final result = deepMergeConfig(
        base,
        override,
        listStrategy: MergeStrategy.append,
      );

      expect(result, {
        'features': ['feature1', 'feature2', 'feature3'],
      });
    });

    test('should replace primitive values', () {
      final base = {'value': 'original'};
      final override = {'value': 'updated'};

      final result = deepMergeConfig(base, override);

      expect(result['value'], 'updated');
    });

    test('should handle null values in override', () {
      final base = {'a': 1, 'b': 2};
      final override = <String, dynamic>{'b': null};

      final result = deepMergeConfig(base, override);

      expect(result, {'a': 1, 'b': null});
    });

    test('should not modify original maps', () {
      final base = {'a': 1};
      final override = {'b': 2};

      deepMergeConfig(base, override);

      expect(base, {'a': 1});
      expect(override, {'b': 2});
    });
  });

  group('mergeAllConfigs', () {
    test('should merge multiple configs in order', () {
      final configs = [
        {'a': 1, 'b': 1, 'c': 1},
        {'b': 2, 'd': 2},
        {'c': 3, 'e': 3},
      ];

      final result = mergeAllConfigs(configs);

      expect(result, {'a': 1, 'b': 2, 'c': 3, 'd': 2, 'e': 3});
    });

    test('should return empty map for empty list', () {
      final result = mergeAllConfigs([]);

      expect(result, isEmpty);
    });

    test('should return first config when only one provided', () {
      final configs = [
        {'a': 1, 'b': 2},
      ];

      final result = mergeAllConfigs(configs);

      expect(result, {'a': 1, 'b': 2});
    });
  });

  group('mergeEnvironmentConfigMaps', () {
    test('should merge default and env configs', () {
      final defaultConfig = {
        'api': {
          'baseUrl': 'https://api.example.com',
          'timeout': 30000,
        },
      };
      final envConfig = {
        'api': {
          'baseUrl': 'https://staging-api.example.com',
        },
      };

      final result = mergeEnvironmentConfigMaps(defaultConfig, envConfig);

      expect(result, {
        'api': {
          'baseUrl': 'https://staging-api.example.com',
          'timeout': 30000,
        },
      });
    });

    test('should return empty map when both configs are null', () {
      final result = mergeEnvironmentConfigMaps(null, null);

      expect(result, isEmpty);
    });

    test('should return default config when env is null', () {
      final defaultConfig = {'a': 1};

      final result = mergeEnvironmentConfigMaps(defaultConfig, null);

      expect(result, {'a': 1});
    });

    test('should return env config when default is null', () {
      final envConfig = {'b': 2};

      final result = mergeEnvironmentConfigMaps(null, envConfig);

      expect(result, {'b': 2});
    });
  });

  group('ConfigMerger (backward compatibility)', () {
    test('deepMerge should delegate to deepMergeConfig', () {
      final base = {'a': 1};
      final override = {'b': 2};

      final result = ConfigMerger.deepMerge(base, override);

      expect(result, {'a': 1, 'b': 2});
    });

    test('mergeAll should delegate to mergeAllConfigs', () {
      final configs = [
        {'a': 1},
        {'b': 2},
      ];

      final result = ConfigMerger.mergeAll(configs);

      expect(result, {'a': 1, 'b': 2});
    });

    test('mergeEnvironmentConfig should delegate to mergeEnvironmentConfigMaps',
        () {
      final defaultConfig = {'a': 1};
      final envConfig = {'b': 2};

      final result =
          ConfigMerger.mergeEnvironmentConfig(defaultConfig, envConfig);

      expect(result, {'a': 1, 'b': 2});
    });
  });
}
