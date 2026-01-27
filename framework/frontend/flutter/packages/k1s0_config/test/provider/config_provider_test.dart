import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/config_loader.dart';
import 'package:k1s0_config/src/config_provider.dart';
import 'package:k1s0_config/src/config_types.dart';
import 'package:k1s0_config/src/config_validator.dart';
import 'package:mocktail/mocktail.dart';

class MockConfigLoader extends Mock implements ConfigLoader {}

class MockConfigValidator extends Mock implements ConfigValidator {}

void main() {
  group('ConfigNotifier', () {
    late MockConfigLoader mockLoader;
    late MockConfigValidator mockValidator;

    setUp(() {
      mockLoader = MockConfigLoader();
      mockValidator = MockConfigValidator();
    });

    test('initial state is ConfigLoading', () {
      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      expect(notifier.state, isA<ConfigLoading>());
    });

    test('load sets state to ConfigLoaded on success', () async {
      const config = AppConfig(appName: 'test-app');
      when(() => mockLoader.load(Environment.dev)).thenAnswer(
        (_) async => const ConfigLoadResult.success(config: config),
      );
      when(() => mockValidator.validate(config))
          .thenReturn(ValidationResult.success());

      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      await notifier.load(Environment.dev);

      expect(notifier.state, isA<ConfigLoaded>());
      final loaded = notifier.state as ConfigLoaded;
      expect(loaded.config, config);
    });

    test('load sets state to ConfigError on load failure', () async {
      when(() => mockLoader.load(Environment.dev)).thenAnswer(
        (_) async => const ConfigLoadResult.failure(
          message: 'File not found',
        ),
      );

      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      await notifier.load(Environment.dev);

      expect(notifier.state, isA<ConfigError>());
      final error = notifier.state as ConfigError;
      expect(error.message, 'File not found');
    });

    test('load sets state to ConfigError on validation failure', () async {
      const config = AppConfig(appName: '');
      when(() => mockLoader.load(Environment.dev)).thenAnswer(
        (_) async => const ConfigLoadResult.success(config: config),
      );
      when(() => mockValidator.validate(config)).thenReturn(
        ValidationResult.failure([
          ConfigValidationError('appName', 'Required'),
        ]),
      );

      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      await notifier.load(Environment.dev);

      expect(notifier.state, isA<ConfigError>());
      final error = notifier.state as ConfigError;
      expect(error.message, contains('validation failed'));
    });

    test('setConfig updates state to ConfigLoaded', () {
      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      const config = AppConfig(appName: 'direct-config');
      notifier.setConfig(config);

      expect(notifier.state, isA<ConfigLoaded>());
      final loaded = notifier.state as ConfigLoaded;
      expect(loaded.config.appName, 'direct-config');
    });

    test('setError updates state to ConfigError', () {
      final notifier = ConfigNotifier(
        loader: mockLoader,
        validator: mockValidator,
      );

      final error = Exception('Test error');
      final stackTrace = StackTrace.current;
      notifier.setError('Test error message', error, stackTrace);

      expect(notifier.state, isA<ConfigError>());
      final errorState = notifier.state as ConfigError;
      expect(errorState.message, 'Test error message');
      expect(errorState.error, error);
      expect(errorState.stackTrace, stackTrace);
    });
  });

  group('appConfigProvider', () {
    test('returns config when state is ConfigLoaded', () {
      const config = AppConfig(appName: 'test-app');
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(appConfigProvider);

      expect(result, config);
    });

    test('returns null when state is not ConfigLoaded', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final result = container.read(appConfigProvider);

      expect(result, isNull);
    });
  });

  group('apiConfigProvider', () {
    test('returns api config when available', () {
      const config = AppConfig(
        appName: 'test-app',
        api: ApiConfig(baseUrl: 'https://api.example.com'),
      );
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(apiConfigProvider);

      expect(result, isNotNull);
      expect(result!.baseUrl, 'https://api.example.com');
    });

    test('returns null when app config is null', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final result = container.read(apiConfigProvider);

      expect(result, isNull);
    });
  });

  group('featureFlagsProvider', () {
    test('returns feature flags when available', () {
      const config = AppConfig(
        appName: 'test-app',
        features: FeatureFlags(flags: {'darkMode': true}),
      );
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(featureFlagsProvider);

      expect(result, isNotNull);
      expect(result!.isEnabled('darkMode'), true);
    });
  });

  group('isFeatureEnabledProvider', () {
    test('returns true for enabled feature', () {
      const config = AppConfig(
        appName: 'test-app',
        features: FeatureFlags(flags: {'feature1': true}),
      );
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(isFeatureEnabledProvider('feature1'));

      expect(result, true);
    });

    test('returns false for disabled feature', () {
      const config = AppConfig(
        appName: 'test-app',
        features: FeatureFlags(flags: {'feature1': false}),
      );
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(isFeatureEnabledProvider('feature1'));

      expect(result, false);
    });

    test('returns false for unknown feature', () {
      const config = AppConfig(
        appName: 'test-app',
        features: FeatureFlags(),
      );
      final container = ProviderContainer(
        overrides: [
          configProvider.overrideWith((ref) {
            final notifier = ConfigNotifier();
            notifier.setConfig(config);
            return notifier;
          }),
        ],
      );
      addTearDown(container.dispose);

      final result = container.read(isFeatureEnabledProvider('unknown'));

      expect(result, false);
    });
  });

  group('ConfigRef extension', () {
    testWidgets('config returns AppConfig when loaded', (tester) async {
      const config = AppConfig(appName: 'test-app');

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            configProvider.overrideWith((ref) {
              final notifier = ConfigNotifier();
              notifier.setConfig(config);
              return notifier;
            }),
          ],
          child: Consumer(
            builder: (context, ref, _) {
              final appConfig = ref.config;
              return Text(
                appConfig?.appName ?? 'null',
                textDirection: TextDirection.ltr,
              );
            },
          ),
        ),
      );

      expect(find.text('test-app'), findsOneWidget);
    });

    testWidgets('requireConfig throws when config not loaded', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: Consumer(
            builder: (context, ref, _) {
              try {
                ref.requireConfig;
                return const Text('no error', textDirection: TextDirection.ltr);
              } catch (e) {
                return Text(
                  'error: ${e.runtimeType}',
                  textDirection: TextDirection.ltr,
                );
              }
            },
          ),
        ),
      );

      expect(find.text('error: StateError'), findsOneWidget);
    });

    testWidgets('isFeatureEnabled returns correct value', (tester) async {
      const config = AppConfig(
        appName: 'test-app',
        features: FeatureFlags(flags: {'darkMode': true}),
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            configProvider.overrideWith((ref) {
              final notifier = ConfigNotifier();
              notifier.setConfig(config);
              return notifier;
            }),
          ],
          child: Consumer(
            builder: (context, ref, _) {
              final isDarkMode = ref.isFeatureEnabled('darkMode');
              return Text(
                isDarkMode.toString(),
                textDirection: TextDirection.ltr,
              );
            },
          ),
        ),
      );

      expect(find.text('true'), findsOneWidget);
    });
  });
}
