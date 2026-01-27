import 'package:freezed_annotation/freezed_annotation.dart';

part 'config_types.freezed.dart';
part 'config_types.g.dart';

/// Environment type for configuration
enum Environment {
  /// Development environment
  dev,

  /// Staging environment
  stg,

  /// Production environment
  prod,
}

/// Extension methods for Environment enum
extension EnvironmentExtension on Environment {
  /// Get the string representation of the environment
  String get value {
    switch (this) {
      case Environment.dev:
        return 'dev';
      case Environment.stg:
        return 'stg';
      case Environment.prod:
        return 'prod';
    }
  }

  /// Parse environment from string
  static Environment fromString(String value) {
    switch (value.toLowerCase()) {
      case 'dev':
      case 'development':
        return Environment.dev;
      case 'stg':
      case 'staging':
        return Environment.stg;
      case 'prod':
      case 'production':
        return Environment.prod;
      default:
        throw ArgumentError('Unknown environment: $value');
    }
  }
}

/// API configuration
@freezed
class ApiConfig with _$ApiConfig {
  /// Creates an API configuration
  const factory ApiConfig({
    /// Base URL for API requests
    required String baseUrl,

    /// Request timeout in milliseconds
    @Default(30000) int timeout,

    /// Number of retry attempts
    @Default(3) int retryCount,

    /// Delay between retries in milliseconds
    @Default(1000) int retryDelay,
  }) = _ApiConfig;

  /// Creates an API configuration from JSON
  factory ApiConfig.fromJson(Map<String, dynamic> json) =>
      _$ApiConfigFromJson(json);
}

/// Authentication configuration
@freezed
class AuthConfig with _$AuthConfig {
  /// Creates an authentication configuration
  const factory AuthConfig({
    /// Whether authentication is enabled
    @Default(true) bool enabled,

    /// Authentication provider type
    @Default('jwt') String provider,

    /// Token refresh threshold in seconds
    @Default(300) int tokenRefreshThreshold,

    /// Token storage type
    @Default('secure') String storage,

    /// OIDC configuration (optional)
    OidcConfig? oidc,
  }) = _AuthConfig;

  /// Creates an authentication configuration from JSON
  factory AuthConfig.fromJson(Map<String, dynamic> json) =>
      _$AuthConfigFromJson(json);
}

/// OIDC configuration
@freezed
class OidcConfig with _$OidcConfig {
  /// Creates an OIDC configuration
  const factory OidcConfig({
    /// Issuer URL
    required String issuer,

    /// Client ID
    required String clientId,

    /// Redirect URI
    required String redirectUri,

    /// Scopes
    @Default(['openid', 'profile', 'email']) List<String> scopes,

    /// Post-logout redirect URI
    String? postLogoutRedirectUri,
  }) = _OidcConfig;

  /// Creates an OIDC configuration from JSON
  factory OidcConfig.fromJson(Map<String, dynamic> json) =>
      _$OidcConfigFromJson(json);
}

/// Logging configuration
@freezed
class LoggingConfig with _$LoggingConfig {
  /// Creates a logging configuration
  const factory LoggingConfig({
    /// Log level
    @Default('info') String level,

    /// Whether console logging is enabled
    @Default(true) bool enableConsole,

    /// Whether remote logging is enabled
    @Default(false) bool enableRemote,

    /// Remote logging endpoint
    String? remoteEndpoint,
  }) = _LoggingConfig;

  /// Creates a logging configuration from JSON
  factory LoggingConfig.fromJson(Map<String, dynamic> json) =>
      _$LoggingConfigFromJson(json);
}

/// Telemetry configuration
@freezed
class TelemetryConfig with _$TelemetryConfig {
  /// Creates a telemetry configuration
  const factory TelemetryConfig({
    /// Whether telemetry is enabled
    @Default(false) bool enabled,

    /// Service name
    @Default('k1s0-flutter') String serviceName,

    /// OTLP endpoint
    String? endpoint,

    /// Sampling rate (0.0 - 1.0)
    @Default(0.1) double sampleRate,
  }) = _TelemetryConfig;

  /// Creates a telemetry configuration from JSON
  factory TelemetryConfig.fromJson(Map<String, dynamic> json) =>
      _$TelemetryConfigFromJson(json);
}

/// Feature flags configuration
@freezed
class FeatureFlags with _$FeatureFlags {
  /// Creates a feature flags configuration
  const factory FeatureFlags({
    /// Feature flag map
    @Default({}) Map<String, bool> flags,
  }) = _FeatureFlags;

  const FeatureFlags._();

  /// Creates feature flags from JSON
  factory FeatureFlags.fromJson(Map<String, dynamic> json) =>
      _$FeatureFlagsFromJson(json);

  /// Check if a feature flag is enabled
  bool isEnabled(String flag) => flags[flag] ?? false;
}

/// Application configuration
@freezed
class AppConfig with _$AppConfig {
  /// Creates an application configuration
  const factory AppConfig({
    /// Environment
    @Default(Environment.dev) Environment env,

    /// Application name
    @Default('k1s0-app') String appName,

    /// Application version
    String? version,

    /// API configuration
    ApiConfig? api,

    /// Authentication configuration
    AuthConfig? auth,

    /// Logging configuration
    LoggingConfig? logging,

    /// Telemetry configuration
    TelemetryConfig? telemetry,

    /// Feature flags
    FeatureFlags? features,

    /// Custom configuration values
    @Default({}) Map<String, dynamic> custom,
  }) = _AppConfig;

  const AppConfig._();

  /// Creates an application configuration from JSON
  factory AppConfig.fromJson(Map<String, dynamic> json) =>
      _$AppConfigFromJson(json);

  /// Check if a feature flag is enabled
  bool isFeatureEnabled(String flag) => features?.isEnabled(flag) ?? false;

  /// Get a custom configuration value
  T? getCustomValue<T>(String key) => custom[key] as T?;
}

/// Configuration load result
@freezed
class ConfigLoadResult with _$ConfigLoadResult {
  /// Creates a successful configuration load result
  const factory ConfigLoadResult.success({
    required AppConfig config,
  }) = ConfigLoadSuccess;

  /// Creates a failed configuration load result
  const factory ConfigLoadResult.failure({
    required String message,
    Object? error,
    StackTrace? stackTrace,
  }) = ConfigLoadFailure;
}
