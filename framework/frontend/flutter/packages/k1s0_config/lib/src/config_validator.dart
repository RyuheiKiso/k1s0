import 'config_schema.dart';
import 'config_types.dart';

/// Validation result
class ValidationResult {
  /// Creates a validation result
  const ValidationResult._({
    required this.isValid,
    required this.errors,
  });

  /// Creates a successful validation result
  factory ValidationResult.success() => const ValidationResult._(
        isValid: true,
        errors: [],
      );

  /// Creates a failed validation result
  factory ValidationResult.failure(List<ConfigValidationError> errors) =>
      ValidationResult._(
        isValid: false,
        errors: errors,
      );

  /// Whether the configuration is valid
  final bool isValid;

  /// List of validation errors
  final List<ConfigValidationError> errors;

  /// Get error messages
  List<String> get errorMessages =>
      errors.map((e) => '${e.field}: ${e.message}').toList();
}

/// Configuration validator
class ConfigValidator {
  /// Creates a configuration validator
  const ConfigValidator({
    this.schema = const _DefaultAppConfigSchema(),
  });

  /// Configuration schema
  final ConfigSchema<AppConfig> schema;

  /// Validate the configuration
  ValidationResult validate(AppConfig config) {
    final errors = schema.validate(config);
    if (errors.isEmpty) {
      return ValidationResult.success();
    }
    return ValidationResult.failure(errors);
  }

  /// Validate and throw if invalid
  void validateOrThrow(AppConfig config) {
    final result = validate(config);
    if (!result.isValid) {
      throw ConfigValidationErrors(result.errors);
    }
  }

  /// Check if the configuration is valid
  bool isValid(AppConfig config) => validate(config).isValid;
}

/// Default schema that performs basic validation
class _DefaultAppConfigSchema extends ConfigSchema<AppConfig> {
  const _DefaultAppConfigSchema();

  @override
  List<ConfigValidationError> validate(AppConfig config) {
    final errors = <ConfigValidationError>[];

    if (config.appName.isEmpty) {
      errors.add(ConfigValidationError('appName', 'App name is required'));
    }

    // Validate API config if present
    if (config.api != null) {
      if (config.api!.baseUrl.isEmpty) {
        errors.add(ConfigValidationError('api.baseUrl', 'Base URL is required'));
      }
    }

    // Validate auth config if present
    if (config.auth != null) {
      if (config.auth!.enabled && config.auth!.provider.isEmpty) {
        errors.add(ConfigValidationError(
          'auth.provider',
          'Provider is required when auth is enabled',
        ));
      }
    }

    return errors;
  }
}

/// Strict configuration validator with full validation
class StrictConfigValidator extends ConfigValidator {
  /// Creates a strict configuration validator
  const StrictConfigValidator()
      : super(schema: const _StrictAppConfigSchema());
}

/// Strict schema that performs comprehensive validation
class _StrictAppConfigSchema extends ConfigSchema<AppConfig> {
  const _StrictAppConfigSchema();

  @override
  List<ConfigValidationError> validate(AppConfig config) {
    return defaultAppConfigSchema.validate(config);
  }
}
