import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'config_loader.dart';
import 'config_types.dart';
import 'config_validator.dart';

/// Configuration state
sealed class ConfigState {
  const ConfigState();
}

/// Configuration is loading
class ConfigLoading extends ConfigState {
  /// Creates a ConfigLoading instance
  const ConfigLoading();
}

/// Configuration loaded successfully
class ConfigLoaded extends ConfigState {
  /// Creates a ConfigLoaded instance
  const ConfigLoaded(this.config);

  /// The loaded configuration
  final AppConfig config;
}

/// Configuration failed to load
class ConfigError extends ConfigState {
  /// Creates a ConfigError instance
  const ConfigError(this.message, [this.error, this.stackTrace]);

  /// Error message
  final String message;

  /// The error object
  final Object? error;

  /// Stack trace
  final StackTrace? stackTrace;
}

/// Configuration notifier for managing configuration state
class ConfigNotifier extends StateNotifier<ConfigState> {
  /// Creates a configuration notifier
  ConfigNotifier({
    ConfigLoader? loader,
    ConfigValidator? validator,
  })  : _loader = loader ?? ConfigLoader(),
        _validator = validator ?? const ConfigValidator(),
        super(const ConfigLoading());

  final ConfigLoader _loader;
  final ConfigValidator _validator;

  /// Load configuration for the specified environment
  Future<void> load(Environment env) async {
    state = const ConfigLoading();

    final result = await _loader.load(env);

    switch (result) {
      case ConfigLoadSuccess(:final config):
        final validation = _validator.validate(config);
        if (validation.isValid) {
          state = ConfigLoaded(config);
        } else {
          state = ConfigError(
            'Configuration validation failed: ${validation.errorMessages.join(', ')}',
          );
        }
      case ConfigLoadFailure(:final message, :final error, :final stackTrace):
        state = ConfigError(message, error, stackTrace);
    }
  }

  /// Set configuration directly (useful for testing)
  void setConfig(AppConfig config) {
    state = ConfigLoaded(config);
  }

  /// Set error state directly (useful for testing)
  void setError(String message, [Object? error, StackTrace? stackTrace]) {
    state = ConfigError(message, error, stackTrace);
  }
}

/// Configuration provider
final configProvider = StateNotifierProvider<ConfigNotifier, ConfigState>(
  (ref) => ConfigNotifier(),
);

/// Convenience provider to get the loaded configuration
final appConfigProvider = Provider<AppConfig?>((ref) {
  final state = ref.watch(configProvider);
  if (state is ConfigLoaded) {
    return state.config;
  }
  return null;
});

/// Provider for API configuration
final apiConfigProvider = Provider<ApiConfig?>(
  (ref) => ref.watch(appConfigProvider)?.api,
);

/// Provider for authentication configuration
final authConfigProvider = Provider<AuthConfig?>(
  (ref) => ref.watch(appConfigProvider)?.auth,
);

/// Provider for logging configuration
final loggingConfigProvider = Provider<LoggingConfig?>(
  (ref) => ref.watch(appConfigProvider)?.logging,
);

/// Provider for telemetry configuration
final telemetryConfigProvider = Provider<TelemetryConfig?>(
  (ref) => ref.watch(appConfigProvider)?.telemetry,
);

/// Provider for feature flags
final featureFlagsProvider = Provider<FeatureFlags?>(
  (ref) => ref.watch(appConfigProvider)?.features,
);

/// Provider to check if a feature flag is enabled
final isFeatureEnabledProvider = Provider.family<bool, String>(
  (ref, flag) => ref.watch(featureFlagsProvider)?.isEnabled(flag) ?? false,
);

/// Configuration widget that initializes configuration on startup
class ConfigScope extends ConsumerStatefulWidget {
  /// Creates a configuration scope widget
  const ConfigScope({
    required this.child,
    required this.environment,
    this.loader,
    this.validator,
    this.onLoading,
    this.onError,
    super.key,
  });

  /// Child widget
  final Widget child;

  /// Environment to load configuration for
  final Environment environment;

  /// Custom configuration loader
  final ConfigLoader? loader;

  /// Custom configuration validator
  final ConfigValidator? validator;

  /// Widget to show while loading configuration
  final Widget Function(BuildContext context)? onLoading;

  /// Widget to show on error
  final Widget Function(BuildContext context, ConfigError error)? onError;

  @override
  ConsumerState<ConfigScope> createState() => _ConfigScopeState();
}

class _ConfigScopeState extends ConsumerState<ConfigScope> {
  @override
  void initState() {
    super.initState();
    _initConfig();
  }

  Future<void> _initConfig() async {
    // Create notifier with custom loader/validator if provided
    if (widget.loader != null || widget.validator != null) {
      ConfigNotifier(
        loader: widget.loader,
        validator: widget.validator,
      );
      ref.read(configProvider.notifier);
    }

    await ref.read(configProvider.notifier).load(widget.environment);
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(configProvider);

    switch (state) {
      case ConfigLoading():
        return widget.onLoading?.call(context) ??
            const Center(child: CircularProgressIndicator());
      case ConfigError():
        return widget.onError?.call(context, state) ??
            Center(
              child: Text(
                'Configuration Error: ${state.message}',
                style: const TextStyle(color: Color(0xFFFF0000)),
              ),
            );
      case ConfigLoaded():
        return widget.child;
    }
  }
}

/// Hook to use configuration in widgets
extension ConfigRef on WidgetRef {
  /// Get the current configuration
  AppConfig? get config => watch(appConfigProvider);

  /// Get the current configuration or throw if not loaded
  AppConfig get requireConfig {
    final config = watch(appConfigProvider);
    if (config == null) {
      throw StateError('Configuration not loaded');
    }
    return config;
  }

  /// Check if a feature flag is enabled
  bool isFeatureEnabled(String flag) => watch(isFeatureEnabledProvider(flag));
}
