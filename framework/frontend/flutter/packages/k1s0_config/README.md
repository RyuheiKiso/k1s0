# k1s0_config

YAML configuration management for k1s0 Flutter applications.

## Features

- YAML configuration file loading from assets
- Type-safe configuration with freezed
- Environment-based configuration merging (default.yaml + {env}.yaml)
- Configuration validation with detailed error messages
- Riverpod-based state management
- ConfigScope widget for easy integration

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_config:
    path: ../packages/k1s0_config
```

## Configuration Files

Place your configuration files in `assets/config/`:

```
assets/
  config/
    default.yaml    # Base configuration
    dev.yaml        # Development overrides
    stg.yaml        # Staging overrides
    prod.yaml       # Production overrides
```

### Example default.yaml

```yaml
appName: my-app
api:
  baseUrl: https://api.example.com
  timeout: 30000
  retryCount: 3
auth:
  enabled: true
  provider: jwt
  tokenRefreshThreshold: 300
logging:
  level: info
  enableConsole: true
features:
  flags:
    newFeature: false
    betaFeature: false
```

### Example dev.yaml

```yaml
api:
  baseUrl: http://localhost:8080
logging:
  level: debug
features:
  flags:
    betaFeature: true
```

## Usage

### Basic Usage with ConfigScope

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_config/k1s0_config.dart';

void main() {
  runApp(
    ProviderScope(
      child: ConfigScope(
        environment: Environment.dev,
        child: const MyApp(),
      ),
    ),
  );
}

class MyApp extends ConsumerWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(appConfigProvider);

    return MaterialApp(
      title: config?.appName ?? 'Loading...',
      home: const HomePage(),
    );
  }
}
```

### Using Configuration in Widgets

```dart
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Get specific configuration sections
    final apiConfig = ref.watch(apiConfigProvider);
    final authConfig = ref.watch(authConfigProvider);

    // Check feature flags
    final isNewFeatureEnabled = ref.watch(isFeatureEnabledProvider('newFeature'));

    // Use the extension for convenience
    final config = ref.requireConfig;

    return Scaffold(
      appBar: AppBar(title: Text(config.appName)),
      body: Column(
        children: [
          Text('API: ${apiConfig?.baseUrl}'),
          if (isNewFeatureEnabled)
            const Text('New feature is enabled!'),
        ],
      ),
    );
  }
}
```

### Custom Loading and Error Handling

```dart
ConfigScope(
  environment: Environment.dev,
  onLoading: (context) => const Center(
    child: Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        CircularProgressIndicator(),
        SizedBox(height: 16),
        Text('Loading configuration...'),
      ],
    ),
  ),
  onError: (context, error) => Center(
    child: Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Icon(Icons.error, color: Colors.red, size: 48),
        const SizedBox(height: 16),
        Text('Failed to load configuration'),
        Text(error.message),
        ElevatedButton(
          onPressed: () => ref.read(configProvider.notifier).load(Environment.dev),
          child: const Text('Retry'),
        ),
      ],
    ),
  ),
  child: const MyApp(),
)
```

### Configuration Validation

```dart
// Use default validator
final validator = ConfigValidator();
final result = validator.validate(config);

if (!result.isValid) {
  for (final error in result.errors) {
    print('${error.field}: ${error.message}');
  }
}

// Use strict validator for comprehensive validation
final strictValidator = StrictConfigValidator();
strictValidator.validateOrThrow(config);
```

### Manual Configuration Loading

```dart
final loader = ConfigLoader(
  options: ConfigLoadOptions(
    configDir: 'assets/config',
    defaultFileName: 'default.yaml',
  ),
);

final result = await loader.load(Environment.dev);

switch (result) {
  case ConfigLoadSuccess(:final config):
    print('Loaded: ${config.appName}');
  case ConfigLoadFailure(:final message):
    print('Error: $message');
}
```

### Testing

```dart
// Use TestConfigLoader for unit tests
final testLoader = ConfigLoaderFactory.createForTest({
  'appName': 'test-app',
  'api': {
    'baseUrl': 'http://test.example.com',
  },
});

final result = await testLoader.load(Environment.dev);
```

## Configuration Types

### AppConfig

Main application configuration containing all sections.

### ApiConfig

- `baseUrl` - Base URL for API requests
- `timeout` - Request timeout in milliseconds (default: 30000)
- `retryCount` - Number of retry attempts (default: 3)
- `retryDelay` - Delay between retries in milliseconds (default: 1000)

### AuthConfig

- `enabled` - Whether authentication is enabled (default: true)
- `provider` - Authentication provider type (jwt, oauth2, oidc, session)
- `tokenRefreshThreshold` - Token refresh threshold in seconds (default: 300)
- `storage` - Token storage type (secure, memory, shared_preferences)
- `oidc` - OIDC configuration (optional)

### LoggingConfig

- `level` - Log level (debug, info, warn, error)
- `enableConsole` - Whether console logging is enabled
- `enableRemote` - Whether remote logging is enabled
- `remoteEndpoint` - Remote logging endpoint

### TelemetryConfig

- `enabled` - Whether telemetry is enabled
- `serviceName` - Service name for telemetry
- `endpoint` - OTLP endpoint
- `sampleRate` - Sampling rate (0.0 - 1.0)

### FeatureFlags

Map of feature flags to boolean values.

## Providers

- `configProvider` - Main configuration state provider
- `appConfigProvider` - Loaded AppConfig or null
- `apiConfigProvider` - API configuration section
- `authConfigProvider` - Auth configuration section
- `loggingConfigProvider` - Logging configuration section
- `telemetryConfigProvider` - Telemetry configuration section
- `featureFlagsProvider` - Feature flags section
- `isFeatureEnabledProvider(flag)` - Check if a specific feature is enabled

## License

MIT
