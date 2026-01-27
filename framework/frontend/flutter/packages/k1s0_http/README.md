# k1s0_http

HTTP client for k1s0 Flutter applications with interceptors, error handling, and tracing.

## Features

- Dio-based HTTP client
- Automatic trace ID generation (W3C Trace Context)
- Authentication token management
- RFC 7807 Problem Details error handling
- Request/response logging
- Retry with exponential backoff
- Type-safe request/response handling

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_http:
    path: ../packages/k1s0_http
```

## Basic Usage

```dart
import 'package:k1s0_http/k1s0_http.dart';

// Create a client
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
  timeout: const Duration(seconds: 30),
);

// Make requests
try {
  final response = await client.get<Map<String, dynamic>>('/users');
  print('Users: ${response.data}');
  print('Trace ID: ${response.traceId}');
} on ApiError catch (e) {
  print('Error: ${e.message}');
  print('Error Code: ${e.errorCode}');
  print('Trace ID: ${e.traceId}');
}
```

## With Authentication

```dart
// Create a token provider
class MyTokenProvider implements TokenProvider {
  @override
  Future<String?> getToken() async {
    // Return the current access token
    return await secureStorage.read(key: 'access_token');
  }

  @override
  Future<void> onTokenRejected() async {
    // Handle 401 errors - clear token, redirect to login, etc.
    await secureStorage.delete(key: 'access_token');
  }
}

// Create client with token provider
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
  tokenProvider: MyTokenProvider(),
);

// Requests will automatically include Authorization header
final response = await client.get<Map<String, dynamic>>('/protected/resource');

// Skip auth for specific requests
final publicResponse = await client.get<Map<String, dynamic>>(
  '/public/resource',
  options: const K1s0RequestOptions(skipAuth: true),
);
```

## Error Handling

The client automatically converts HTTP errors to `ApiError` with RFC 7807 Problem Details support:

```dart
try {
  final response = await client.post<Map<String, dynamic>>(
    '/users',
    data: {'name': ''},
  );
} on ApiError catch (e) {
  switch (e.kind) {
    case ApiErrorKind.validation:
      // Handle validation errors
      for (final fieldError in e.fieldErrors) {
        print('${fieldError.field}: ${fieldError.message}');
      }
      break;

    case ApiErrorKind.authentication:
      // Redirect to login
      break;

    case ApiErrorKind.notFound:
      // Show not found message
      break;

    default:
      // Show generic error
      print(e.localizedMessage); // User-friendly message in Japanese
  }
}
```

## Retry Configuration

```dart
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
  retryPolicy: const RetryPolicy(
    maxAttempts: 3,
    initialDelay: Duration(seconds: 1),
    maxDelay: Duration(seconds: 30),
    backoffMultiplier: 2.0,
    retryStatusCodes: [502, 503, 504],
    retryOnTimeout: true,
  ),
);

// Enable retry for specific requests
final response = await client.get<Map<String, dynamic>>(
  '/unreliable/endpoint',
  options: const K1s0RequestOptions(retry: true),
);
```

## Request Options

```dart
final response = await client.get<Map<String, dynamic>>(
  '/users',
  options: K1s0RequestOptions(
    // Additional headers
    headers: {'X-Custom-Header': 'value'},

    // Query parameters
    queryParameters: {'page': 1, 'limit': 10},

    // Custom timeout
    timeout: 60000, // milliseconds

    // Skip authentication
    skipAuth: true,

    // Enable retry
    retry: true,

    // Custom trace ID
    traceId: 'custom-trace-id',

    // Extra data for interceptors
    extra: {'key': 'value'},
  ),
);
```

## Logging

```dart
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
  logLevel: HttpLogLevel.headers, // none, basic, headers, body
  logger: (message) {
    // Custom logger
    debugPrint(message);
  },
);
```

## Error Callbacks

```dart
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
  onError: (error) {
    // Log all errors
    analytics.logError(error);
  },
  onAuthError: (error) {
    // Handle auth errors globally
    authService.logout();
  },
);
```

## Custom Interceptors

```dart
final client = K1s0HttpClientFactory.create(
  baseUrl: 'https://api.example.com',
);

// Add custom interceptors
client.dio.interceptors.add(
  InterceptorsWrapper(
    onRequest: (options, handler) {
      // Modify request
      options.headers['X-Custom'] = 'value';
      handler.next(options);
    },
    onResponse: (response, handler) {
      // Modify response
      handler.next(response);
    },
  ),
);
```

## API Reference

### K1s0HttpClient

| Method | Description |
|--------|-------------|
| `get<T>(path, options)` | GET request |
| `post<T>(path, data, options)` | POST request |
| `put<T>(path, data, options)` | PUT request |
| `patch<T>(path, data, options)` | PATCH request |
| `delete<T>(path, data, options)` | DELETE request |
| `close()` | Close the client |

### ApiError

| Property | Type | Description |
|----------|------|-------------|
| `kind` | `ApiErrorKind` | Error classification |
| `message` | `String` | Error message |
| `statusCode` | `int?` | HTTP status code |
| `errorCode` | `String?` | Server error code |
| `traceId` | `String?` | Trace ID for debugging |
| `problemDetails` | `ProblemDetails?` | Full RFC 7807 response |
| `fieldErrors` | `List<FieldError>` | Validation errors |
| `isRetryable` | `bool` | Whether the error is retryable |
| `requiresAuthentication` | `bool` | Whether re-authentication is needed |
| `localizedMessage` | `String` | User-friendly message |

### ApiErrorKind

- `validation` - Input validation error (400)
- `authentication` - Authentication required (401)
- `authorization` - Access denied (403)
- `notFound` - Resource not found (404)
- `conflict` - Resource conflict (409)
- `rateLimit` - Too many requests (429)
- `dependency` - Upstream service error (502/503)
- `temporary` - Temporary error (5xx)
- `timeout` - Request timeout
- `network` - Network error
- `connection` - Connection failed
- `cancelled` - Request cancelled
- `unknown` - Unknown error

## License

MIT
