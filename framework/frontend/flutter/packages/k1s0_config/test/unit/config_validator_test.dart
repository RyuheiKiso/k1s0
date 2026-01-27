import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/config_types.dart';
import 'package:k1s0_config/src/config_validator.dart';

void main() {
  group('ValidationResult', () {
    test('success creates valid result', () {
      final result = ValidationResult.success();

      expect(result.isValid, true);
      expect(result.errors, isEmpty);
      expect(result.errorMessages, isEmpty);
    });

    test('failure creates invalid result', () {
      final errors = [
        ConfigValidationError('field1', 'Error 1'),
        ConfigValidationError('field2', 'Error 2'),
      ];
      final result = ValidationResult.failure(errors);

      expect(result.isValid, false);
      expect(result.errors, hasLength(2));
    });

    test('errorMessages returns formatted messages', () {
      final errors = [
        ConfigValidationError('appName', 'App name is required'),
        ConfigValidationError('api.baseUrl', 'Base URL is required'),
      ];
      final result = ValidationResult.failure(errors);

      expect(result.errorMessages, [
        'appName: App name is required',
        'api.baseUrl: Base URL is required',
      ]);
    });
  });

  group('ConfigValidator', () {
    late ConfigValidator validator;

    setUp(() {
      validator = const ConfigValidator();
    });

    test('validates valid config', () {
      const config = AppConfig(
        appName: 'my-app',
        api: ApiConfig(baseUrl: 'https://api.example.com'),
      );

      final result = validator.validate(config);

      expect(result.isValid, true);
    });

    test('fails when appName is empty', () {
      const config = AppConfig(appName: '');

      final result = validator.validate(config);

      expect(result.isValid, false);
      expect(result.errors.any((e) => e.field == 'appName'), true);
    });

    test('fails when api.baseUrl is empty', () {
      const config = AppConfig(
        appName: 'my-app',
        api: ApiConfig(baseUrl: ''),
      );

      final result = validator.validate(config);

      expect(result.isValid, false);
      expect(result.errors.any((e) => e.field == 'api.baseUrl'), true);
    });

    test('fails when auth is enabled but provider is empty', () {
      const config = AppConfig(
        appName: 'my-app',
        auth: AuthConfig(enabled: true, provider: ''),
      );

      final result = validator.validate(config);

      expect(result.isValid, false);
      expect(result.errors.any((e) => e.field == 'auth.provider'), true);
    });

    test('passes when auth is disabled with empty provider', () {
      const config = AppConfig(
        appName: 'my-app',
        auth: AuthConfig(enabled: false, provider: ''),
      );

      final result = validator.validate(config);

      expect(result.isValid, true);
    });

    test('isValid returns correct boolean', () {
      const validConfig = AppConfig(appName: 'my-app');
      const invalidConfig = AppConfig(appName: '');

      expect(validator.isValid(validConfig), true);
      expect(validator.isValid(invalidConfig), false);
    });

    test('validateOrThrow throws for invalid config', () {
      const invalidConfig = AppConfig(appName: '');

      expect(
        () => validator.validateOrThrow(invalidConfig),
        throwsA(isA<ConfigValidationErrors>()),
      );
    });

    test('validateOrThrow does not throw for valid config', () {
      const validConfig = AppConfig(appName: 'my-app');

      expect(
        () => validator.validateOrThrow(validConfig),
        returnsNormally,
      );
    });
  });

  group('StrictConfigValidator', () {
    late StrictConfigValidator validator;

    setUp(() {
      validator = const StrictConfigValidator();
    });

    test('validates using strict schema', () {
      const config = AppConfig(
        appName: 'my-app',
      );

      final result = validator.validate(config);

      expect(result, isA<ValidationResult>());
    });
  });
}
