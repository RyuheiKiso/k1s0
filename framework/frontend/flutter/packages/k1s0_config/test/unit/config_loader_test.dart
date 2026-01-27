import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/config_loader.dart';
import 'package:k1s0_config/src/config_types.dart';

void main() {
  group('ConfigLoadOptions', () {
    test('creates with default values', () {
      const options = ConfigLoadOptions();

      expect(options.configDir, 'assets/config');
      expect(options.defaultFileName, 'default.yaml');
      expect(options.envFilePattern, '{env}.yaml');
      expect(options.allowMissingDefault, true);
      expect(options.allowMissingEnv, true);
    });

    test('creates with custom values', () {
      const options = ConfigLoadOptions(
        configDir: 'config',
        defaultFileName: 'base.yaml',
        envFilePattern: 'config-{env}.yaml',
        allowMissingDefault: false,
        allowMissingEnv: false,
      );

      expect(options.configDir, 'config');
      expect(options.defaultFileName, 'base.yaml');
      expect(options.envFilePattern, 'config-{env}.yaml');
      expect(options.allowMissingDefault, false);
      expect(options.allowMissingEnv, false);
    });
  });

  group('TestConfigLoader', () {
    test('loads test config successfully', () async {
      final testConfig = {
        'appName': 'test-app',
        'api': {
          'baseUrl': 'https://test-api.example.com',
        },
      };
      final loader = TestConfigLoader(testConfig);

      final result = await loader.load(Environment.dev);

      expect(result, isA<ConfigLoadSuccess>());
      final success = result as ConfigLoadSuccess;
      expect(success.config.appName, 'test-app');
      expect(success.config.env, Environment.dev);
    });

    test('sets environment in loaded config', () async {
      final testConfig = {'appName': 'test-app'};
      final loader = TestConfigLoader(testConfig);

      final devResult = await loader.load(Environment.dev);
      final prodResult = await loader.load(Environment.prod);

      final devConfig = (devResult as ConfigLoadSuccess).config;
      final prodConfig = (prodResult as ConfigLoadSuccess).config;

      expect(devConfig.env, Environment.dev);
      expect(prodConfig.env, Environment.prod);
    });

    test('returns failure for invalid config', () async {
      // Invalid config that cannot be parsed
      final testConfig = <String, dynamic>{
        'api': 'invalid', // api should be an object
      };
      final loader = TestConfigLoader(testConfig);

      final result = await loader.load(Environment.dev);

      expect(result, isA<ConfigLoadFailure>());
    });
  });

  group('createConfigLoader', () {
    test('creates ConfigLoader with default options', () {
      final loader = createConfigLoader();

      expect(loader, isA<ConfigLoader>());
      expect(loader.options.configDir, 'assets/config');
    });

    test('creates ConfigLoader with custom options', () {
      const options = ConfigLoadOptions(configDir: 'custom/config');
      final loader = createConfigLoader(options: options);

      expect(loader.options.configDir, 'custom/config');
    });
  });

  group('createTestConfigLoader', () {
    test('creates TestConfigLoader', () {
      final config = {'appName': 'test'};
      final loader = createTestConfigLoader(config);

      expect(loader, isA<TestConfigLoader>());
    });
  });

  group('ConfigLoaderFactory (backward compatibility)', () {
    test('create returns ConfigLoader', () {
      final loader = ConfigLoaderFactory.create();

      expect(loader, isA<ConfigLoader>());
    });

    test('createForTest returns TestConfigLoader', () {
      final config = {'appName': 'test'};
      final loader = ConfigLoaderFactory.createForTest(config);

      expect(loader, isA<TestConfigLoader>());
    });
  });
}
