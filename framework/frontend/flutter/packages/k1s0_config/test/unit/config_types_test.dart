import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/config_types.dart';

void main() {
  group('Environment', () {
    test('value returns correct string representation', () {
      expect(Environment.dev.value, 'dev');
      expect(Environment.stg.value, 'stg');
      expect(Environment.prod.value, 'prod');
    });

    test('fromString parses dev variants', () {
      expect(EnvironmentExtension.fromString('dev'), Environment.dev);
      expect(EnvironmentExtension.fromString('development'), Environment.dev);
      expect(EnvironmentExtension.fromString('DEV'), Environment.dev);
    });

    test('fromString parses stg variants', () {
      expect(EnvironmentExtension.fromString('stg'), Environment.stg);
      expect(EnvironmentExtension.fromString('staging'), Environment.stg);
      expect(EnvironmentExtension.fromString('STG'), Environment.stg);
    });

    test('fromString parses prod variants', () {
      expect(EnvironmentExtension.fromString('prod'), Environment.prod);
      expect(EnvironmentExtension.fromString('production'), Environment.prod);
      expect(EnvironmentExtension.fromString('PROD'), Environment.prod);
    });

    test('fromString throws ArgumentError for unknown environment', () {
      expect(
        () => EnvironmentExtension.fromString('unknown'),
        throwsArgumentError,
      );
    });
  });

  group('ApiConfig', () {
    test('creates with default values', () {
      const config = ApiConfig(baseUrl: 'https://api.example.com');

      expect(config.baseUrl, 'https://api.example.com');
      expect(config.timeout, 30000);
      expect(config.retryCount, 3);
      expect(config.retryDelay, 1000);
    });

    test('creates with custom values', () {
      const config = ApiConfig(
        baseUrl: 'https://api.example.com',
        timeout: 60000,
        retryCount: 5,
        retryDelay: 2000,
      );

      expect(config.timeout, 60000);
      expect(config.retryCount, 5);
      expect(config.retryDelay, 2000);
    });

    test('fromJson creates correct instance', () {
      final json = {
        'baseUrl': 'https://api.example.com',
        'timeout': 45000,
        'retryCount': 2,
        'retryDelay': 500,
      };

      final config = ApiConfig.fromJson(json);

      expect(config.baseUrl, 'https://api.example.com');
      expect(config.timeout, 45000);
      expect(config.retryCount, 2);
      expect(config.retryDelay, 500);
    });

    test('toJson returns correct map', () {
      const config = ApiConfig(
        baseUrl: 'https://api.example.com',
        timeout: 30000,
      );

      final json = config.toJson();

      expect(json['baseUrl'], 'https://api.example.com');
      expect(json['timeout'], 30000);
    });
  });

  group('AuthConfig', () {
    test('creates with default values', () {
      const config = AuthConfig();

      expect(config.enabled, true);
      expect(config.provider, 'jwt');
      expect(config.tokenRefreshThreshold, 300);
      expect(config.storage, 'secure');
      expect(config.oidc, isNull);
    });

    test('creates with OIDC config', () {
      const config = AuthConfig(
        oidc: OidcConfig(
          issuer: 'https://auth.example.com',
          clientId: 'client-123',
          redirectUri: 'myapp://callback',
        ),
      );

      expect(config.oidc, isNotNull);
      expect(config.oidc!.issuer, 'https://auth.example.com');
    });
  });

  group('LoggingConfig', () {
    test('creates with default values', () {
      const config = LoggingConfig();

      expect(config.level, 'info');
      expect(config.enableConsole, true);
      expect(config.enableRemote, false);
      expect(config.remoteEndpoint, isNull);
    });
  });

  group('TelemetryConfig', () {
    test('creates with default values', () {
      const config = TelemetryConfig();

      expect(config.enabled, false);
      expect(config.serviceName, 'k1s0-flutter');
      expect(config.endpoint, isNull);
      expect(config.sampleRate, 0.1);
    });
  });

  group('FeatureFlags', () {
    test('creates with empty flags by default', () {
      const flags = FeatureFlags();

      expect(flags.flags, isEmpty);
    });

    test('isEnabled returns true for enabled flags', () {
      const flags = FeatureFlags(flags: {'feature1': true, 'feature2': false});

      expect(flags.isEnabled('feature1'), true);
      expect(flags.isEnabled('feature2'), false);
    });

    test('isEnabled returns false for unknown flags', () {
      const flags = FeatureFlags(flags: {'feature1': true});

      expect(flags.isEnabled('unknown'), false);
    });
  });

  group('AppConfig', () {
    test('creates with default values', () {
      const config = AppConfig();

      expect(config.env, Environment.dev);
      expect(config.appName, 'k1s0-app');
      expect(config.version, isNull);
      expect(config.api, isNull);
      expect(config.custom, isEmpty);
    });

    test('creates with all configurations', () {
      const config = AppConfig(
        env: Environment.prod,
        appName: 'my-app',
        version: '1.0.0',
        api: ApiConfig(baseUrl: 'https://api.example.com'),
        auth: AuthConfig(),
        logging: LoggingConfig(),
        telemetry: TelemetryConfig(),
        features: FeatureFlags(flags: {'darkMode': true}),
        custom: {'customKey': 'customValue'},
      );

      expect(config.env, Environment.prod);
      expect(config.appName, 'my-app');
      expect(config.version, '1.0.0');
      expect(config.api, isNotNull);
      expect(config.auth, isNotNull);
      expect(config.logging, isNotNull);
      expect(config.telemetry, isNotNull);
      expect(config.features, isNotNull);
      expect(config.custom['customKey'], 'customValue');
    });

    test('isFeatureEnabled delegates to features', () {
      const config = AppConfig(
        features: FeatureFlags(flags: {'feature1': true}),
      );

      expect(config.isFeatureEnabled('feature1'), true);
      expect(config.isFeatureEnabled('feature2'), false);
    });

    test('isFeatureEnabled returns false when features is null', () {
      const config = AppConfig();

      expect(config.isFeatureEnabled('anyFeature'), false);
    });

    test('getCustomValue returns correct value', () {
      const config = AppConfig(
        custom: {
          'stringValue': 'test',
          'intValue': 42,
        },
      );

      expect(config.getCustomValue<String>('stringValue'), 'test');
      expect(config.getCustomValue<int>('intValue'), 42);
    });

    test('getCustomValue returns null for missing key', () {
      const config = AppConfig();

      expect(config.getCustomValue<String>('missing'), isNull);
    });
  });

  group('ConfigLoadResult', () {
    test('success creates successful result', () {
      const config = AppConfig();
      const result = ConfigLoadResult.success(config: config);

      expect(result, isA<ConfigLoadSuccess>());
      final success = result as ConfigLoadSuccess;
      expect(success.config, config);
    });

    test('failure creates failure result', () {
      const result = ConfigLoadResult.failure(
        message: 'Config file not found',
      );

      expect(result, isA<ConfigLoadFailure>());
      final failure = result as ConfigLoadFailure;
      expect(failure.message, 'Config file not found');
      expect(failure.error, isNull);
    });

    test('failure with error creates failure result', () {
      final error = Exception('File not found');
      final stackTrace = StackTrace.current;
      final result = ConfigLoadResult.failure(
        message: 'Config file not found',
        error: error,
        stackTrace: stackTrace,
      );

      expect(result, isA<ConfigLoadFailure>());
      final failure = result as ConfigLoadFailure;
      expect(failure.error, error);
      expect(failure.stackTrace, stackTrace);
    });
  });
}
