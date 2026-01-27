import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../persistence/state_storage.dart';
import 'app_state.dart';

/// Notifier for global app state.
class AppStateNotifier extends Notifier<AppState> {
  @override
  AppState build() => const AppState();

  /// Marks the app as initialized.
  void setInitialized() {
    state = state.copyWith(initialized: true);
  }

  /// Sets the loading state.
  void setLoading({required bool loading}) {
    state = state.copyWith(loading: loading);
  }

  /// Sets the environment.
  void setEnvironment(String environment) {
    state = state.copyWith(environment: environment);
  }

  /// Sets the locale.
  void setLocale(String locale) {
    state = state.copyWith(locale: locale);
  }

  /// Sets the dark mode.
  void setDarkMode({required bool isDarkMode}) {
    state = state.copyWith(isDarkMode: isDarkMode);
  }

  /// Sets a feature flag.
  void setFeatureFlag(String flag, {required bool enabled}) {
    state = state.withFeatureFlag(flag, enabled: enabled);
  }

  /// Sets multiple feature flags.
  void setFeatureFlags(Map<String, bool> flags) {
    state = state.copyWith(
      featureFlags: {...state.featureFlags, ...flags},
    );
  }

  /// Sets metadata.
  void setMetadata(String key, Object? value) {
    state = state.withMetadata(key, value);
  }

  /// Resets the state to initial.
  void reset() {
    state = const AppState();
  }
}

/// Provider for global app state.
final appStateProvider = NotifierProvider<AppStateNotifier, AppState>(
  AppStateNotifier.new,
);

/// Provider for app initialized state.
final appInitializedProvider = Provider<bool>(
  (ref) => ref.watch(appStateProvider.select((s) => s.initialized)),
);

/// Provider for app loading state.
final appLoadingProvider = Provider<bool>(
  (ref) => ref.watch(appStateProvider.select((s) => s.loading)),
);

/// Provider for current environment.
final environmentProvider = Provider<String>(
  (ref) => ref.watch(appStateProvider.select((s) => s.environment)),
);

/// Provider for current locale.
final localeProvider = Provider<String>(
  (ref) => ref.watch(appStateProvider.select((s) => s.locale)),
);

/// Provider for dark mode state.
final isDarkModeProvider = Provider<bool>(
  (ref) => ref.watch(appStateProvider.select((s) => s.isDarkMode)),
);

/// Provider for feature flags.
final featureFlagsProvider = Provider<Map<String, bool>>(
  (ref) => ref.watch(appStateProvider.select((s) => s.featureFlags)),
);

/// Provider to check if a specific feature is enabled.
final isFeatureEnabledProvider = Provider.family<bool, String>(
  (ref, flag) => ref.watch(featureFlagsProvider)[flag] ?? false,
);

/// Notifier for user preferences with persistence.
class UserPreferencesNotifier extends Notifier<UserPreferences> {
  StateStorage? _storage;
  static const _storageKey = 'user_preferences';

  /// Initializes the notifier with storage.
  void initialize(StateStorage storage) {
    _storage = storage;
    _loadPreferences();
  }

  Future<void> _loadPreferences() async {
    if (_storage == null) return;
    final json = await _storage!.read<Map<String, dynamic>>(_storageKey);
    if (json != null) {
      state = UserPreferences.fromJson(json);
    }
  }

  Future<void> _savePreferences() async {
    if (_storage == null) return;
    await _storage!.write(_storageKey, state.toJson());
  }

  @override
  UserPreferences build() => const UserPreferences();

  /// Sets the theme mode.
  void setThemeMode(String mode) {
    state = state.copyWith(themeMode: mode);
    _savePreferences();
  }

  /// Sets the preferred locale.
  void setPreferredLocale(String? locale) {
    state = state.copyWith(preferredLocale: locale);
    _savePreferences();
  }

  /// Sets notifications enabled.
  void setNotificationsEnabled({required bool enabled}) {
    state = state.copyWith(notificationsEnabled: enabled);
    _savePreferences();
  }

  /// Sets analytics consent.
  void setAnalyticsConsent({required bool consent}) {
    state = state.copyWith(analyticsConsent: consent);
    _savePreferences();
  }

  /// Sets a custom preference.
  void setCustomPreference(String key, Object? value) {
    state = state.copyWith(
      custom: {...state.custom, key: value},
    );
    _savePreferences();
  }

  /// Gets a custom preference.
  T? getCustomPreference<T>(String key) => state.custom[key] as T?;

  /// Clears all preferences.
  Future<void> clear() async {
    state = const UserPreferences();
    await _storage?.delete(_storageKey);
  }
}

/// Provider for user preferences.
final userPreferencesProvider =
    NotifierProvider<UserPreferencesNotifier, UserPreferences>(
  UserPreferencesNotifier.new,
);

/// Provider for theme mode preference.
final themeModePreferenceProvider = Provider<String>(
  (ref) => ref.watch(userPreferencesProvider.select((s) => s.themeMode)),
);

/// Notifier for navigation state.
class NavigationStateNotifier extends Notifier<NavigationState> {
  @override
  NavigationState build() => const NavigationState();

  /// Updates the current navigation state.
  void navigate(
    String path, {
    Map<String, String>? params,
    Map<String, String>? queryParams,
  }) {
    state = state.push(path, params: params, queryParams: queryParams);
  }

  /// Goes back to the previous route.
  void goBack() {
    state = state.pop();
  }

  /// Resets the navigation history.
  void reset() {
    state = const NavigationState();
  }
}

/// Provider for navigation state.
final navigationStateProvider =
    NotifierProvider<NavigationStateNotifier, NavigationState>(
  NavigationStateNotifier.new,
);

/// Provider for current route path.
final currentPathProvider = Provider<String>(
  (ref) => ref.watch(navigationStateProvider.select((s) => s.currentPath)),
);

/// Notifier for connectivity state.
class ConnectivityStateNotifier extends Notifier<ConnectivityState> {
  @override
  ConnectivityState build() => const ConnectivityState();

  /// Updates the connectivity state.
  void updateConnectivity({
    required bool isConnected,
    String? connectionType,
  }) {
    state = state.copyWith(
      isConnected: isConnected,
      connectionType: connectionType ?? state.connectionType,
      lastChecked: DateTime.now(),
    );
  }

  /// Sets the device as connected.
  void setConnected(String connectionType) {
    updateConnectivity(isConnected: true, connectionType: connectionType);
  }

  /// Sets the device as disconnected.
  void setDisconnected() {
    updateConnectivity(isConnected: false, connectionType: 'none');
  }
}

/// Provider for connectivity state.
final connectivityStateProvider =
    NotifierProvider<ConnectivityStateNotifier, ConnectivityState>(
  ConnectivityStateNotifier.new,
);

/// Provider for checking if connected.
final isConnectedProvider = Provider<bool>(
  (ref) => ref.watch(connectivityStateProvider.select((s) => s.isConnected)),
);
