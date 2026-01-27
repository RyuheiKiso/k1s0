import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_state/src/global/app_state.dart';

void main() {
  group('AppState', () {
    test('creates with default values', () {
      const state = AppState();

      expect(state.initialized, false);
      expect(state.loading, false);
      expect(state.environment, 'development');
      expect(state.locale, 'en');
      expect(state.isDarkMode, false);
      expect(state.featureFlags, isEmpty);
      expect(state.metadata, isEmpty);
    });

    test('creates with custom values', () {
      const state = AppState(
        initialized: true,
        loading: true,
        environment: 'production',
        locale: 'ja',
        isDarkMode: true,
        featureFlags: {'darkMode': true},
        metadata: {'version': '1.0.0'},
      );

      expect(state.initialized, true);
      expect(state.loading, true);
      expect(state.environment, 'production');
      expect(state.locale, 'ja');
      expect(state.isDarkMode, true);
      expect(state.featureFlags['darkMode'], true);
      expect(state.metadata['version'], '1.0.0');
    });

    test('fromJson creates correct instance', () {
      final json = {
        'initialized': true,
        'loading': false,
        'environment': 'staging',
        'locale': 'en',
        'isDarkMode': true,
        'featureFlags': {'feature1': true},
        'metadata': {'key': 'value'},
      };

      final state = AppState.fromJson(json);

      expect(state.initialized, true);
      expect(state.environment, 'staging');
      expect(state.isDarkMode, true);
      expect(state.featureFlags['feature1'], true);
    });

    test('toJson returns correct map', () {
      const state = AppState(
        initialized: true,
        environment: 'production',
      );

      final json = state.toJson();

      expect(json['initialized'], true);
      expect(json['environment'], 'production');
    });

    test('copyWith creates new instance with updated values', () {
      const original = AppState();

      final updated = original.copyWith(
        initialized: true,
        environment: 'production',
      );

      expect(original.initialized, false);
      expect(updated.initialized, true);
      expect(updated.environment, 'production');
      expect(updated.locale, original.locale);
    });
  });

  group('AppStateExtensions', () {
    test('isFeatureEnabled returns true for enabled flag', () {
      const state = AppState(
        featureFlags: {'feature1': true, 'feature2': false},
      );

      expect(state.isFeatureEnabled('feature1'), true);
      expect(state.isFeatureEnabled('feature2'), false);
    });

    test('isFeatureEnabled returns false for unknown flag', () {
      const state = AppState(featureFlags: {'feature1': true});

      expect(state.isFeatureEnabled('unknown'), false);
    });

    test('withFeatureFlag adds/updates flag', () {
      const state = AppState(featureFlags: {'existing': true});

      final updated = state.withFeatureFlag('newFlag', enabled: true);

      expect(updated.featureFlags['existing'], true);
      expect(updated.featureFlags['newFlag'], true);
    });

    test('withMetadata adds metadata', () {
      const state = AppState(metadata: {'existing': 'value'});

      final updated = state.withMetadata('newKey', 'newValue');

      expect(updated.metadata['existing'], 'value');
      expect(updated.metadata['newKey'], 'newValue');
    });

    test('getMetadata returns correct value', () {
      const state = AppState(metadata: {'stringKey': 'value', 'intKey': 42});

      expect(state.getMetadata<String>('stringKey'), 'value');
      expect(state.getMetadata<int>('intKey'), 42);
    });

    test('getMetadata returns null for missing key', () {
      const state = AppState();

      expect(state.getMetadata<String>('missing'), isNull);
    });
  });

  group('UserPreferences', () {
    test('creates with default values', () {
      const prefs = UserPreferences();

      expect(prefs.themeMode, 'system');
      expect(prefs.preferredLocale, isNull);
      expect(prefs.notificationsEnabled, true);
      expect(prefs.analyticsConsent, false);
      expect(prefs.custom, isEmpty);
    });

    test('creates with custom values', () {
      const prefs = UserPreferences(
        themeMode: 'dark',
        preferredLocale: 'ja',
        notificationsEnabled: false,
        analyticsConsent: true,
        custom: {'customKey': 'customValue'},
      );

      expect(prefs.themeMode, 'dark');
      expect(prefs.preferredLocale, 'ja');
      expect(prefs.notificationsEnabled, false);
      expect(prefs.analyticsConsent, true);
      expect(prefs.custom['customKey'], 'customValue');
    });

    test('fromJson creates correct instance', () {
      final json = {
        'themeMode': 'light',
        'preferredLocale': 'en',
        'notificationsEnabled': true,
        'analyticsConsent': false,
        'custom': {},
      };

      final prefs = UserPreferences.fromJson(json);

      expect(prefs.themeMode, 'light');
      expect(prefs.preferredLocale, 'en');
    });
  });

  group('NavigationState', () {
    test('creates with default values', () {
      const state = NavigationState();

      expect(state.currentPath, '/');
      expect(state.previousPath, isNull);
      expect(state.params, isEmpty);
      expect(state.queryParams, isEmpty);
      expect(state.history, isEmpty);
    });

    test('canGoBack returns false when history is empty', () {
      const state = NavigationState();

      expect(state.canGoBack, false);
    });

    test('canGoBack returns false when only one item in history', () {
      const state = NavigationState(history: ['/']);

      expect(state.canGoBack, false);
    });

    test('canGoBack returns true when multiple items in history', () {
      const state = NavigationState(history: ['/', '/page1']);

      expect(state.canGoBack, true);
    });

    test('push adds to history', () {
      const state = NavigationState(currentPath: '/', history: ['/']);

      final updated = state.push('/page1');

      expect(updated.currentPath, '/page1');
      expect(updated.previousPath, '/');
      expect(updated.history, ['/', '/page1']);
    });

    test('push with params and queryParams', () {
      const state = NavigationState(currentPath: '/', history: ['/']);

      final updated = state.push(
        '/users/:id',
        params: {'id': '123'},
        queryParams: {'tab': 'profile'},
      );

      expect(updated.params['id'], '123');
      expect(updated.queryParams['tab'], 'profile');
    });

    test('pop removes from history', () {
      const state = NavigationState(
        currentPath: '/page1',
        history: ['/', '/page1'],
      );

      final updated = state.pop();

      expect(updated.currentPath, '/');
      expect(updated.previousPath, '/page1');
      expect(updated.history, ['/']);
    });

    test('pop does nothing when cannot go back', () {
      const state = NavigationState(currentPath: '/', history: ['/']);

      final updated = state.pop();

      expect(updated, state);
    });
  });

  group('ConnectivityState', () {
    test('creates with default values', () {
      const state = ConnectivityState();

      expect(state.isConnected, true);
      expect(state.connectionType, 'unknown');
      expect(state.lastChecked, isNull);
    });

    test('creates with custom values', () {
      final now = DateTime.now();
      final state = ConnectivityState(
        isConnected: false,
        connectionType: 'wifi',
        lastChecked: now,
      );

      expect(state.isConnected, false);
      expect(state.connectionType, 'wifi');
      expect(state.lastChecked, now);
    });
  });
}
