import 'config_types.dart';

/// Configuration validation error
class ConfigValidationError implements Exception {
  /// Creates a configuration validation error
  ConfigValidationError(this.field, this.message, [this.value]);

  /// The field that failed validation
  final String field;

  /// The error message
  final String message;

  /// The invalid value
  final Object? value;

  @override
  String toString() =>
      'ConfigValidationError: $field - $message (value: $value)';
}

/// Configuration validation errors collection
class ConfigValidationErrors implements Exception {
  /// Creates a configuration validation errors collection
  ConfigValidationErrors(this.errors);

  /// List of validation errors
  final List<ConfigValidationError> errors;

  /// Whether there are any errors
  bool get hasErrors => errors.isNotEmpty;

  /// Get all error messages
  List<String> get messages => errors.map((e) => e.toString()).toList();

  @override
  String toString() => 'ConfigValidationErrors: ${messages.join(', ')}';
}

/// Configuration schema for validation
abstract class ConfigSchema<T> {
  /// Creates a configuration schema
  const ConfigSchema();

  /// Validate the configuration
  List<ConfigValidationError> validate(T config);

  /// Check if the configuration is valid
  bool isValid(T config) => validate(config).isEmpty;

  /// Validate and throw if invalid
  void validateOrThrow(T config) {
    final errors = validate(config);
    if (errors.isNotEmpty) {
      throw ConfigValidationErrors(errors);
    }
  }
}

/// API configuration schema
class ApiConfigSchema extends ConfigSchema<ApiConfig> {
  @override
  List<ConfigValidationError> validate(ApiConfig config) {
    final errors = <ConfigValidationError>[];

    if (config.baseUrl.isEmpty) {
      errors.add(ConfigValidationError('baseUrl', 'Base URL is required'));
    } else if (!(Uri.tryParse(config.baseUrl)?.hasScheme ?? false)) {
      errors.add(
        ConfigValidationError(
          'baseUrl',
          'Base URL must be a valid URL with scheme',
          config.baseUrl,
        ),
      );
    }

    if (config.timeout <= 0) {
      errors.add(
        ConfigValidationError(
          'timeout',
          'Timeout must be positive',
          config.timeout,
        ),
      );
    }

    if (config.retryCount < 0) {
      errors.add(
        ConfigValidationError(
          'retryCount',
          'Retry count must be non-negative',
          config.retryCount,
        ),
      );
    }

    if (config.retryDelay < 0) {
      errors.add(
        ConfigValidationError(
          'retryDelay',
          'Retry delay must be non-negative',
          config.retryDelay,
        ),
      );
    }

    return errors;
  }
}

/// Authentication configuration schema
class AuthConfigSchema extends ConfigSchema<AuthConfig> {
  @override
  List<ConfigValidationError> validate(AuthConfig config) {
    final errors = <ConfigValidationError>[];

    final validProviders = ['jwt', 'oauth2', 'oidc', 'session'];
    if (!validProviders.contains(config.provider)) {
      errors.add(
        ConfigValidationError(
          'provider',
          'Provider must be one of: ${validProviders.join(', ')}',
          config.provider,
        ),
      );
    }

    if (config.tokenRefreshThreshold <= 0) {
      errors.add(
        ConfigValidationError(
          'tokenRefreshThreshold',
          'Token refresh threshold must be positive',
          config.tokenRefreshThreshold,
        ),
      );
    }

    final validStorages = ['secure', 'memory', 'shared_preferences'];
    if (!validStorages.contains(config.storage)) {
      errors.add(
        ConfigValidationError(
          'storage',
          'Storage must be one of: ${validStorages.join(', ')}',
          config.storage,
        ),
      );
    }

    // Validate OIDC config if present
    if (config.oidc != null) {
      errors.addAll(_validateOidcConfig(config.oidc!));
    }

    return errors;
  }

  List<ConfigValidationError> _validateOidcConfig(OidcConfig config) {
    final errors = <ConfigValidationError>[];

    if (config.issuer.isEmpty) {
      errors.add(ConfigValidationError('oidc.issuer', 'Issuer is required'));
    } else if (!(Uri.tryParse(config.issuer)?.hasScheme ?? false)) {
      errors.add(
        ConfigValidationError(
          'oidc.issuer',
          'Issuer must be a valid URL',
          config.issuer,
        ),
      );
    }

    if (config.clientId.isEmpty) {
      errors
          .add(ConfigValidationError('oidc.clientId', 'Client ID is required'));
    }

    if (config.redirectUri.isEmpty) {
      errors.add(
        ConfigValidationError(
          'oidc.redirectUri',
          'Redirect URI is required',
        ),
      );
    }

    return errors;
  }
}

/// Logging configuration schema
class LoggingConfigSchema extends ConfigSchema<LoggingConfig> {
  @override
  List<ConfigValidationError> validate(LoggingConfig config) {
    final errors = <ConfigValidationError>[];

    final validLevels = ['debug', 'info', 'warn', 'error'];
    if (!validLevels.contains(config.level.toLowerCase())) {
      errors.add(
        ConfigValidationError(
          'level',
          'Log level must be one of: ${validLevels.join(', ')}',
          config.level,
        ),
      );
    }

    if (config.enableRemote && (config.remoteEndpoint?.isEmpty ?? true)) {
      errors.add(
        ConfigValidationError(
          'remoteEndpoint',
          'Remote endpoint is required when remote logging is enabled',
        ),
      );
    }

    return errors;
  }
}

/// Telemetry configuration schema
class TelemetryConfigSchema extends ConfigSchema<TelemetryConfig> {
  @override
  List<ConfigValidationError> validate(TelemetryConfig config) {
    final errors = <ConfigValidationError>[];

    if (config.sampleRate < 0 || config.sampleRate > 1) {
      errors.add(
        ConfigValidationError(
          'sampleRate',
          'Sample rate must be between 0 and 1',
          config.sampleRate,
        ),
      );
    }

    if (config.serviceName.isEmpty) {
      errors.add(
        ConfigValidationError(
          'serviceName',
          'Service name is required',
        ),
      );
    }

    if (config.enabled && (config.endpoint?.isEmpty ?? true)) {
      errors.add(
        ConfigValidationError(
          'endpoint',
          'Endpoint is required when telemetry is enabled',
        ),
      );
    }

    return errors;
  }
}

/// Application configuration schema
class AppConfigSchema extends ConfigSchema<AppConfig> {
  /// Creates an application configuration schema
  AppConfigSchema({
    this.apiSchema,
    this.authSchema,
    this.loggingSchema,
    this.telemetrySchema,
  });

  /// API configuration schema
  final ApiConfigSchema? apiSchema;

  /// Authentication configuration schema
  final AuthConfigSchema? authSchema;

  /// Logging configuration schema
  final LoggingConfigSchema? loggingSchema;

  /// Telemetry configuration schema
  final TelemetryConfigSchema? telemetrySchema;

  @override
  List<ConfigValidationError> validate(AppConfig config) {
    final errors = <ConfigValidationError>[];

    if (config.appName.isEmpty) {
      errors.add(ConfigValidationError('appName', 'App name is required'));
    }

    if (config.api != null && apiSchema != null) {
      errors.addAll(apiSchema!.validate(config.api!));
    }

    if (config.auth != null && authSchema != null) {
      errors.addAll(authSchema!.validate(config.auth!));
    }

    if (config.logging != null && loggingSchema != null) {
      errors.addAll(loggingSchema!.validate(config.logging!));
    }

    if (config.telemetry != null && telemetrySchema != null) {
      errors.addAll(telemetrySchema!.validate(config.telemetry!));
    }

    return errors;
  }
}

/// Default application configuration schema with all sub-schemas
final defaultAppConfigSchema = AppConfigSchema(
  apiSchema: ApiConfigSchema(),
  authSchema: AuthConfigSchema(),
  loggingSchema: LoggingConfigSchema(),
  telemetrySchema: TelemetryConfigSchema(),
);
