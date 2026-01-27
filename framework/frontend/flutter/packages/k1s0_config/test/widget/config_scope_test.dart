import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_config/src/config_loader.dart';
import 'package:k1s0_config/src/config_provider.dart';
import 'package:k1s0_config/src/config_types.dart';

void main() {
  group('ConfigScope widget', () {
    testWidgets('shows loading widget while loading', (tester) async {
      // Use a completer to control when the config loads
      final loader = TestConfigLoader({'appName': 'test-app'});

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.dev,
              loader: loader,
              onLoading: (context) => const Text('Loading...'),
              child: const Text('Loaded'),
            ),
          ),
        ),
      );

      // Initially should show loading
      expect(find.text('Loading...'), findsOneWidget);

      // Wait for the config to load
      await tester.pumpAndSettle();

      // Now should show the child
      expect(find.text('Loaded'), findsOneWidget);
    });

    testWidgets('shows default loading indicator when onLoading is null',
        (tester) async {
      final loader = TestConfigLoader({'appName': 'test-app'});

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.dev,
              loader: loader,
              child: const Text('Loaded'),
            ),
          ),
        ),
      );

      // Should show CircularProgressIndicator by default
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('shows child widget when config is loaded', (tester) async {
      final loader = TestConfigLoader({
        'appName': 'test-app',
        'api': {'baseUrl': 'https://api.example.com'},
      });

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.dev,
              loader: loader,
              child: Consumer(
                builder: (context, ref, _) {
                  final config = ref.watch(appConfigProvider);
                  return Text('App: ${config?.appName ?? "null"}');
                },
              ),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('App: test-app'), findsOneWidget);
    });

    testWidgets('shows error widget when config loading fails', (tester) async {
      // Create a loader that will return an invalid config
      final loader = TestConfigLoader(<String, dynamic>{
        'api': 'invalid', // This will cause parsing to fail
      });

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.dev,
              loader: loader,
              onError: (context, error) => Text('Error: ${error.message}'),
              child: const Text('Loaded'),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.textContaining('Error:'), findsOneWidget);
    });

    testWidgets('shows default error widget when onError is null',
        (tester) async {
      final loader = TestConfigLoader(<String, dynamic>{
        'api': 'invalid',
      });

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.dev,
              loader: loader,
              child: const Text('Loaded'),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.textContaining('Configuration Error:'), findsOneWidget);
    });

    testWidgets('uses correct environment', (tester) async {
      final loader = TestConfigLoader({'appName': 'test-app'});

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: ConfigScope(
              environment: Environment.prod,
              loader: loader,
              child: Consumer(
                builder: (context, ref, _) {
                  final config = ref.watch(appConfigProvider);
                  return Text('Env: ${config?.env.value ?? "null"}');
                },
              ),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('Env: prod'), findsOneWidget);
    });
  });

  group('ConfigState sealed class', () {
    test('ConfigLoading is a ConfigState', () {
      const state = ConfigLoading();
      expect(state, isA<ConfigState>());
    });

    test('ConfigLoaded is a ConfigState', () {
      const config = AppConfig(appName: 'test');
      const state = ConfigLoaded(config);
      expect(state, isA<ConfigState>());
      expect(state.config, config);
    });

    test('ConfigError is a ConfigState', () {
      const state = ConfigError('Test error');
      expect(state, isA<ConfigState>());
      expect(state.message, 'Test error');
    });

    test('ConfigError can hold error and stackTrace', () {
      final error = Exception('Test');
      final stackTrace = StackTrace.current;
      final state = ConfigError('Test error', error, stackTrace);

      expect(state.error, error);
      expect(state.stackTrace, stackTrace);
    });
  });
}
