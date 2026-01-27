import 'package:freezed_annotation/freezed_annotation.dart';

part 'app_state.freezed.dart';
part 'app_state.g.dart';

/// Global application state.
///
/// Contains the shared state across the entire application.
@freezed
class AppState with _$AppState {
  const factory AppState({
    /// Whether the app has been initialized.
    @Default(false) bool initialized,

    /// Whether the app is in a loading state.
    @Default(false) bool loading,

    /// Current environment (e.g., 'development', 'staging', 'production').
    @Default('development') String environment,

    /// Current locale code (e.g., 'en', 'ja').
    @Default('en') String locale,

    /// Whether dark mode is enabled.
    @Default(false) bool isDarkMode,

    /// Feature flags.
    @Default({}) Map<String, bool> featureFlags,

    /// Custom metadata.
    @Default({}) Map<String, dynamic> metadata,
  }) = _AppState;

  factory AppState.fromJson(Map<String, dynamic> json) =>
      _$AppStateFromJson(json);
}

/// Extension methods for AppState.
extension AppStateExtensions on AppState {
  /// Returns true if a feature flag is enabled.
  bool isFeatureEnabled(String flag) {
    return featureFlags[flag] ?? false;
  }

  /// Creates a copy with the feature flag set.
  AppState withFeatureFlag(String flag, bool enabled) {
    return copyWith(
      featureFlags: {...featureFlags, flag: enabled},
    );
  }

  /// Creates a copy with metadata set.
  AppState withMetadata(String key, dynamic value) {
    return copyWith(
      metadata: {...metadata, key: value},
    );
  }

  /// Gets metadata value.
  T? getMetadata<T>(String key) {
    return metadata[key] as T?;
  }
}

/// User preferences state.
@freezed
class UserPreferences with _$UserPreferences {
  const factory UserPreferences({
    /// Preferred theme mode ('light', 'dark', 'system').
    @Default('system') String themeMode,

    /// Preferred locale.
    String? preferredLocale,

    /// Notification preferences.
    @Default(true) bool notificationsEnabled,

    /// Analytics consent.
    @Default(false) bool analyticsConsent,

    /// Custom preferences.
    @Default({}) Map<String, dynamic> custom,
  }) = _UserPreferences;

  factory UserPreferences.fromJson(Map<String, dynamic> json) =>
      _$UserPreferencesFromJson(json);
}

/// Navigation state for tracking navigation history.
@freezed
class NavigationState with _$NavigationState {
  const factory NavigationState({
    /// Current route path.
    @Default('/') String currentPath,

    /// Previous route path.
    String? previousPath,

    /// Route parameters.
    @Default({}) Map<String, String> params,

    /// Query parameters.
    @Default({}) Map<String, String> queryParams,

    /// Navigation history stack.
    @Default([]) List<String> history,
  }) = _NavigationState;

  factory NavigationState.fromJson(Map<String, dynamic> json) =>
      _$NavigationStateFromJson(json);
}

/// Extension methods for NavigationState.
extension NavigationStateExtensions on NavigationState {
  /// Whether there is a previous route to go back to.
  bool get canGoBack => history.length > 1;

  /// Pushes a new route to the history.
  NavigationState push(String path, {
    Map<String, String>? params,
    Map<String, String>? queryParams,
  }) {
    return copyWith(
      previousPath: currentPath,
      currentPath: path,
      params: params ?? {},
      queryParams: queryParams ?? {},
      history: [...history, path],
    );
  }

  /// Pops the current route from the history.
  NavigationState pop() {
    if (!canGoBack) return this;
    final newHistory = [...history]..removeLast();
    return copyWith(
      previousPath: currentPath,
      currentPath: newHistory.last,
      history: newHistory,
    );
  }
}

/// Connectivity state.
@freezed
class ConnectivityState with _$ConnectivityState {
  const factory ConnectivityState({
    /// Whether the device is connected to the internet.
    @Default(true) bool isConnected,

    /// Connection type ('wifi', 'mobile', 'none').
    @Default('unknown') String connectionType,

    /// Last time connectivity was checked.
    DateTime? lastChecked,
  }) = _ConnectivityState;

  factory ConnectivityState.fromJson(Map<String, dynamic> json) =>
      _$ConnectivityStateFromJson(json);
}
